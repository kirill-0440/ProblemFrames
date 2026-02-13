use crate::ast::*;
use crate::language::{
    parse_domain_kind, parse_domain_role, parse_frame_type, parse_phenomenon_type,
};
use anyhow::{anyhow, Result};
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
            _ => {}
        }
    }

    Ok(problem)
}
