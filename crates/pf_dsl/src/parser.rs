use crate::ast::*;
use crate::language::{
    parse_domain_kind, parse_domain_role, parse_frame_type, parse_phenomenon_type,
};
use anyhow::{anyhow, Result};
use pest::error::InputLocation;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "problem_frames.pest"]
pub struct PFParser;

fn pair_to_span(pair: &Pair<Rule>) -> Span {
    let span = pair.as_span();
    Span {
        start: span.start(),
        end: span.end(),
    }
}

fn next_inner<'a>(pair: Pair<'a, Rule>, expected: &str) -> Result<Pair<'a, Rule>> {
    pair.into_inner()
        .next()
        .ok_or_else(|| anyhow!("missing {expected}"))
}

fn parse_assertion_stmt(assertion_pair: Pair<'_, Rule>) -> Result<Assertion> {
    let span = pair_to_span(&assertion_pair);
    let mut inner = assertion_pair.into_inner();
    let text_pair = inner
        .next()
        .ok_or_else(|| anyhow!("missing assertion text"))?;
    let language = inner
        .next()
        .map(|pair| pair.as_str().trim_start_matches('@').to_string());

    Ok(Assertion {
        text: text_pair.as_str().trim_matches('"').to_string(),
        language,
        span,
    })
}

fn parse_assertion_set(pair: Pair<'_, Rule>, scope: AssertionScope) -> Result<AssertionSet> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| anyhow!("missing assertion set name"))?;
    let mut assertions = Vec::new();
    for assertion_pair in inner {
        if assertion_pair.as_rule() == Rule::assertion_stmt {
            assertions.push(parse_assertion_stmt(assertion_pair)?);
        }
    }

    Ok(AssertionSet {
        name: name_pair.as_str().to_string(),
        scope,
        assertions,
        span,
        source_path: None,
    })
}

pub fn parse_error_diagnostic(input: &str) -> Option<(Span, String)> {
    match PFParser::parse(Rule::program, input) {
        Ok(_) => None,
        Err(err) => {
            let span = match err.location {
                InputLocation::Pos(pos) => Span {
                    start: pos,
                    end: (pos + 1).min(input.len()),
                },
                InputLocation::Span((start, end)) => Span {
                    start,
                    end: end.min(input.len()),
                },
            };
            Some((span, err.to_string()))
        }
    }
}

pub fn parse(input: &str) -> Result<Problem> {
    let mut pairs = PFParser::parse(Rule::program, input)?;
    let program_pair = pairs
        .next()
        .ok_or_else(|| anyhow!("program did not produce a parse tree"))?;
    let problem_span = pair_to_span(&program_pair);

    let mut problem = Problem {
        name: String::new(),
        span: problem_span,
        imports: vec![],
        domains: vec![],
        interfaces: vec![],
        requirements: vec![],
        assertion_sets: vec![],
        correctness_arguments: vec![],
    };

    for pair in program_pair.into_inner() {
        let span = pair_to_span(&pair);
        match pair.as_rule() {
            Rule::import_decl => {
                let path_literal = next_inner(pair, "import path literal")?.as_str();
                problem
                    .imports
                    .push(path_literal.trim_matches('"').to_string());
            }
            Rule::problem_decl => {
                let name_pair = next_inner(pair, "problem name")?;
                problem.name = name_pair.as_str().trim().to_string();
            }
            Rule::domain_decl => {
                let mut inner = pair.into_inner();
                let name = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing domain name"))?
                    .as_str()
                    .to_string();
                let kind_pair = inner.next().ok_or_else(|| anyhow!("missing domain kind"))?;
                let role_pair = inner.next().ok_or_else(|| anyhow!("missing domain role"))?;

                problem.domains.push(Domain {
                    name,
                    kind: parse_domain_kind(kind_pair.as_str()),
                    role: parse_domain_role(role_pair.as_str()),
                    span,
                    source_path: None,
                });
            }
            Rule::interface_decl => {
                let mut inner = pair.into_inner();
                let name = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing interface name"))?
                    .as_str()
                    .trim_matches('"')
                    .to_string();
                let connects_pair = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing interface connects list"))?;
                let shared_pair = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing shared phenomena block"))?;

                let mut connects = Vec::new();
                for domain_ident in connects_pair.into_inner() {
                    if domain_ident.as_rule() == Rule::identifier {
                        connects.push(Reference {
                            name: domain_ident.as_str().to_string(),
                            span: pair_to_span(&domain_ident),
                        });
                    }
                }

                let mut phenomena = Vec::new();
                for phen_pair in shared_pair.into_inner() {
                    if phen_pair.as_rule() != Rule::phenomenon {
                        continue;
                    }

                    let p_span = pair_to_span(&phen_pair);
                    let mut p_inner = phen_pair.into_inner();
                    let name_pair = p_inner
                        .next()
                        .ok_or_else(|| anyhow!("missing phenomenon name"))?;
                    let type_pair = p_inner
                        .next()
                        .ok_or_else(|| anyhow!("missing phenomenon type"))?;
                    let from_pair = p_inner
                        .next()
                        .ok_or_else(|| anyhow!("missing phenomenon source domain"))?;
                    let to_pair = p_inner
                        .next()
                        .ok_or_else(|| anyhow!("missing phenomenon target domain"))?;
                    let controlled_by_pair = p_inner
                        .next()
                        .ok_or_else(|| anyhow!("missing controlledBy domain"))?;

                    phenomena.push(Phenomenon {
                        name: name_pair.as_str().to_string(),
                        type_: parse_phenomenon_type(type_pair.as_str())
                            .unwrap_or(PhenomenonType::Event),
                        from: Reference {
                            name: from_pair.as_str().to_string(),
                            span: pair_to_span(&from_pair),
                        },
                        to: Reference {
                            name: to_pair.as_str().to_string(),
                            span: pair_to_span(&to_pair),
                        },
                        controlled_by: Reference {
                            name: controlled_by_pair.as_str().to_string(),
                            span: pair_to_span(&controlled_by_pair),
                        },
                        span: p_span,
                    });
                }

                problem.interfaces.push(Interface {
                    name,
                    connects,
                    shared_phenomena: phenomena,
                    span,
                    source_path: None,
                });
            }
            Rule::requirement_decl => {
                let mut inner = pair.into_inner();
                let name = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing requirement name"))?
                    .as_str()
                    .trim_matches('"')
                    .to_string();
                let req_body_pair = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing requirement body"))?;

                let mut req = Requirement {
                    name,
                    frame: FrameType::Custom(String::new()),
                    phenomena: vec![],
                    constraint: String::new(),
                    constrains: None,
                    reference: None,
                    span,
                    source_path: None,
                };

                for field in req_body_pair.into_inner() {
                    match field.as_rule() {
                        Rule::frame_type => {
                            let type_str = next_inner(field, "frame type identifier")?.as_str();
                            req.frame = parse_frame_type(type_str);
                        }
                        Rule::constraint => {
                            let s = next_inner(field, "constraint literal")?.as_str();
                            req.constraint = s.trim_matches('"').to_string();
                        }
                        Rule::constrains => {
                            let ident_pair = next_inner(field, "constrains domain")?;
                            req.constrains = Some(Reference {
                                name: ident_pair.as_str().to_string(),
                                span: pair_to_span(&ident_pair),
                            });
                        }
                        Rule::reference => {
                            let ident_pair = next_inner(field, "reference domain")?;
                            req.reference = Some(Reference {
                                name: ident_pair.as_str().to_string(),
                                span: pair_to_span(&ident_pair),
                            });
                        }
                        _ => {}
                    }
                }

                problem.requirements.push(req);
            }
            Rule::world_properties_decl => {
                problem
                    .assertion_sets
                    .push(parse_assertion_set(pair, AssertionScope::WorldProperties)?);
            }
            Rule::specification_decl => {
                problem
                    .assertion_sets
                    .push(parse_assertion_set(pair, AssertionScope::Specification)?);
            }
            Rule::requirement_assertions_decl => {
                problem.assertion_sets.push(parse_assertion_set(
                    pair,
                    AssertionScope::RequirementAssertions,
                )?);
            }
            Rule::correctness_argument_decl => {
                let mut inner = pair.into_inner();
                let name_pair = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing correctness argument name"))?;
                let prove_pair = inner
                    .next()
                    .ok_or_else(|| anyhow!("missing prove statement"))?;
                let mut prove_inner = prove_pair.into_inner();
                let specification_set = prove_inner
                    .next()
                    .ok_or_else(|| anyhow!("missing specification set reference"))?;
                let world_set = prove_inner
                    .next()
                    .ok_or_else(|| anyhow!("missing world properties set reference"))?;
                let requirement_set = prove_inner
                    .next()
                    .ok_or_else(|| anyhow!("missing requirement set reference"))?;

                problem.correctness_arguments.push(CorrectnessArgument {
                    name: name_pair.as_str().to_string(),
                    specification_set: specification_set.as_str().to_string(),
                    world_set: world_set.as_str().to_string(),
                    requirement_set: requirement_set.as_str().to_string(),
                    span,
                    source_path: None,
                });
            }
            _ => {}
        }
    }

    Ok(problem)
}
