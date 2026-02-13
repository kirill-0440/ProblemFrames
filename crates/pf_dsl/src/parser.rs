use crate::ast::*;
use crate::language::{
    parse_domain_kind, parse_domain_role, parse_frame_type, parse_phenomenon_type,
};
use anyhow::{anyhow, Result};
use pest::error::InputLocation;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use std::collections::HashSet;

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

#[derive(Debug, Clone)]
struct ParseDiagnostic {
    span: Span,
    message: String,
}

impl ParseDiagnostic {
    fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }
}

fn span_from_pest_error(err: &pest::error::Error<Rule>, input: &str) -> Span {
    match err.location {
        InputLocation::Pos(pos) => Span {
            start: pos.min(input.len()),
            end: (pos + 1).min(input.len()),
        },
        InputLocation::Span((start, end)) => Span {
            start: start.min(input.len()),
            end: end.min(input.len()),
        },
    }
}

fn next_inner<'a>(
    pair: Pair<'a, Rule>,
    expected: &str,
    span: Span,
) -> std::result::Result<Pair<'a, Rule>, ParseDiagnostic> {
    pair.into_inner()
        .next()
        .ok_or_else(|| ParseDiagnostic::new(span, format!("missing {expected}")))
}

fn parse_assertion_stmt(
    assertion_pair: Pair<'_, Rule>,
) -> std::result::Result<Assertion, ParseDiagnostic> {
    let span = pair_to_span(&assertion_pair);
    let mut inner = assertion_pair.into_inner();
    let text_pair = inner
        .next()
        .ok_or_else(|| ParseDiagnostic::new(span, "missing assertion text"))?;
    let language = inner
        .next()
        .map(|pair| pair.as_str().trim_start_matches('@').to_string());

    Ok(Assertion {
        text: text_pair.as_str().trim_matches('"').to_string(),
        language,
        span,
    })
}

fn parse_assertion_set(
    pair: Pair<'_, Rule>,
    scope: AssertionScope,
) -> std::result::Result<AssertionSet, ParseDiagnostic> {
    let span = pair_to_span(&pair);
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| ParseDiagnostic::new(span, "missing assertion set name"))?;
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
    match parse_internal(input) {
        Ok(_) => None,
        Err(err) => Some((err.span, err.message)),
    }
}

pub fn parse(input: &str) -> Result<Problem> {
    parse_internal(input).map_err(|err| anyhow!("{}", err.message))
}

fn parse_internal(input: &str) -> std::result::Result<Problem, ParseDiagnostic> {
    let mut pairs = PFParser::parse(Rule::program, input)
        .map_err(|err| ParseDiagnostic::new(span_from_pest_error(&err, input), err.to_string()))?;
    let program_pair = pairs.next().ok_or_else(|| {
        ParseDiagnostic::new(
            Span { start: 0, end: 0 },
            "program did not produce a parse tree",
        )
    })?;
    let problem_span = pair_to_span(&program_pair);

    let mut problem = Problem {
        name: String::new(),
        span: problem_span,
        imports: vec![],
        domains: vec![],
        interfaces: vec![],
        requirements: vec![],
        subproblems: vec![],
        assertion_sets: vec![],
        correctness_arguments: vec![],
    };
    let mut has_problem_decl = false;

    for pair in program_pair.into_inner() {
        let span = pair_to_span(&pair);
        match pair.as_rule() {
            Rule::import_decl => {
                let path_literal = next_inner(pair, "import path literal", span)?.as_str();
                problem
                    .imports
                    .push(path_literal.trim_matches('"').to_string());
            }
            Rule::problem_decl => {
                if has_problem_decl {
                    return Err(ParseDiagnostic::new(
                        span,
                        "multiple problem declarations are not allowed",
                    ));
                }
                let name_pair = next_inner(pair, "problem name", span)?;
                problem.name = name_pair.as_str().trim().to_string();
                has_problem_decl = true;
            }
            Rule::domain_decl => {
                let mut inner = pair.into_inner();
                let name = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing domain name"))?
                    .as_str()
                    .to_string();
                let kind_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing domain kind"))?;
                let role_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing domain role"))?;

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
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing interface name"))?
                    .as_str()
                    .trim_matches('"')
                    .to_string();
                let connects_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing interface connects list"))?;
                let shared_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing shared phenomena block"))?;

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
                        .ok_or_else(|| ParseDiagnostic::new(p_span, "missing phenomenon name"))?;
                    let type_pair = p_inner
                        .next()
                        .ok_or_else(|| ParseDiagnostic::new(p_span, "missing phenomenon type"))?;
                    let from_pair = p_inner.next().ok_or_else(|| {
                        ParseDiagnostic::new(p_span, "missing phenomenon source domain")
                    })?;
                    let to_pair = p_inner.next().ok_or_else(|| {
                        ParseDiagnostic::new(p_span, "missing phenomenon target domain")
                    })?;
                    let controlled_by_pair = p_inner.next().ok_or_else(|| {
                        ParseDiagnostic::new(p_span, "missing controlledBy domain")
                    })?;

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
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing requirement name"))?
                    .as_str()
                    .trim_matches('"')
                    .to_string();
                let req_body_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing requirement body"))?;
                let req_body_span = pair_to_span(&req_body_pair);

                let mut required_fields = HashSet::new();
                let mut has_frame = false;

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
                    let rule = field.as_rule();
                    let field_span = pair_to_span(&field);
                    let field_seen = match rule {
                        Rule::frame_type => "frame",
                        Rule::constraint => "constraint",
                        Rule::constrains => "constrains",
                        Rule::reference => "reference",
                        _ => "unknown",
                    };

                    if field_seen != "unknown" && !required_fields.insert(field_seen) {
                        return Err(ParseDiagnostic::new(
                            field_span,
                            format!(
                                "requirement '{}' has duplicate field '{}'",
                                req.name, field_seen
                            ),
                        ));
                    }

                    match field.as_rule() {
                        Rule::frame_type => {
                            let frame_value_pair = next_inner(field, "frame type", field_span)?;
                            let type_str = match frame_value_pair.as_rule() {
                                Rule::string_literal => {
                                    frame_value_pair.as_str().trim_matches('"').to_string()
                                }
                                _ => frame_value_pair.as_str().to_string(),
                            };
                            if type_str.trim().is_empty() {
                                return Err(ParseDiagnostic::new(
                                    pair_to_span(&frame_value_pair),
                                    format!(
                                        "requirement '{}' has invalid empty frame value",
                                        req.name
                                    ),
                                ));
                            }
                            has_frame = true;
                            req.frame = parse_frame_type(type_str.as_str());
                        }
                        Rule::constraint => {
                            let s = next_inner(field, "constraint literal", field_span)?.as_str();
                            req.constraint = s.trim_matches('"').to_string();
                        }
                        Rule::constrains => {
                            let ident_pair = next_inner(field, "constrains domain", field_span)?;
                            req.constrains = Some(Reference {
                                name: ident_pair.as_str().to_string(),
                                span: pair_to_span(&ident_pair),
                            });
                        }
                        Rule::reference => {
                            let ident_pair = next_inner(field, "reference domain", field_span)?;
                            req.reference = Some(Reference {
                                name: ident_pair.as_str().to_string(),
                                span: pair_to_span(&ident_pair),
                            });
                        }
                        _ => {}
                    }
                }

                if !has_frame {
                    return Err(ParseDiagnostic::new(
                        req_body_span,
                        format!(
                            "requirement '{}' is missing required field 'frame:'",
                            req.name
                        ),
                    ));
                }

                problem.requirements.push(req);
            }
            Rule::subproblem_decl => {
                let mut inner = pair.into_inner();
                let name = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing subproblem name"))?
                    .as_str()
                    .to_string();
                let body = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing subproblem body"))?;
                let body_span = pair_to_span(&body);

                let mut required_fields = HashSet::new();
                let mut has_machine = false;
                let mut has_participants = false;
                let mut has_requirements = false;
                let mut seen_requirements = HashSet::new();
                let mut seen_participants = HashSet::new();

                let mut subproblem = Subproblem {
                    name,
                    machine: None,
                    participants: vec![],
                    requirements: vec![],
                    span,
                    source_path: None,
                };

                for field in body.into_inner() {
                    let field_seen = match field.as_rule() {
                        Rule::subproblem_machine => "machine",
                        Rule::subproblem_participants => "participants",
                        Rule::subproblem_requirements => "requirements",
                        _ => "unknown",
                    };
                    let field_span = pair_to_span(&field);

                    if field_seen != "unknown" && !required_fields.insert(field_seen) {
                        return Err(ParseDiagnostic::new(
                            field_span,
                            format!(
                                "subproblem '{}' has duplicate field '{}'",
                                subproblem.name, field_seen
                            ),
                        ));
                    }

                    match field.as_rule() {
                        Rule::subproblem_machine => {
                            let machine_pair =
                                next_inner(field, "subproblem machine domain", field_span)?;
                            has_machine = true;
                            subproblem.machine = Some(Reference {
                                name: machine_pair.as_str().to_string(),
                                span: pair_to_span(&machine_pair),
                            });
                        }
                        Rule::subproblem_participants => {
                            has_participants = true;
                            let list_pair =
                                next_inner(field, "subproblem participants list", field_span)?;
                            for participant_pair in list_pair.into_inner() {
                                if participant_pair.as_rule() == Rule::identifier {
                                    let name = participant_pair.as_str().to_string();
                                    let span = pair_to_span(&participant_pair);
                                    if !seen_participants.insert(name.clone()) {
                                        return Err(ParseDiagnostic::new(
                                            span,
                                            format!(
                                                "subproblem '{}' has duplicate participant '{}'",
                                                subproblem.name, name
                                            ),
                                        ));
                                    }
                                    subproblem.participants.push(Reference { name, span });
                                }
                            }
                        }
                        Rule::subproblem_requirements => {
                            has_requirements = true;
                            let list_pair =
                                next_inner(field, "subproblem requirements list", field_span)?;
                            for requirement_pair in list_pair.into_inner() {
                                if requirement_pair.as_rule() == Rule::string_literal {
                                    let name =
                                        requirement_pair.as_str().trim_matches('"').to_string();
                                    let span = pair_to_span(&requirement_pair);
                                    if !seen_requirements.insert(name.clone()) {
                                        return Err(ParseDiagnostic::new(
                                            span,
                                            format!(
                                                "subproblem '{}' has duplicate requirement reference '{}'",
                                                subproblem.name, name
                                            ),
                                        ));
                                    }
                                    subproblem.requirements.push(Reference { name, span });
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if !has_machine {
                    return Err(ParseDiagnostic::new(
                        body_span,
                        format!(
                            "subproblem '{}' is missing required field 'machine:'",
                            subproblem.name
                        ),
                    ));
                }
                if !has_participants {
                    return Err(ParseDiagnostic::new(
                        body_span,
                        format!(
                            "subproblem '{}' is missing required field 'participants:'",
                            subproblem.name
                        ),
                    ));
                }
                if !has_requirements {
                    return Err(ParseDiagnostic::new(
                        body_span,
                        format!(
                            "subproblem '{}' is missing required field 'requirements:'",
                            subproblem.name
                        ),
                    ));
                }

                problem.subproblems.push(subproblem);
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
                let name_pair = inner.next().ok_or_else(|| {
                    ParseDiagnostic::new(span, "missing correctness argument name")
                })?;
                let prove_pair = inner
                    .next()
                    .ok_or_else(|| ParseDiagnostic::new(span, "missing prove statement"))?;
                let mut prove_inner = prove_pair.into_inner();
                let specification_set = prove_inner.next().ok_or_else(|| {
                    ParseDiagnostic::new(span, "missing specification set reference")
                })?;
                let world_set = prove_inner.next().ok_or_else(|| {
                    ParseDiagnostic::new(span, "missing world properties set reference")
                })?;
                let requirement_set = prove_inner.next().ok_or_else(|| {
                    ParseDiagnostic::new(span, "missing requirement set reference")
                })?;

                problem.correctness_arguments.push(CorrectnessArgument {
                    name: name_pair.as_str().to_string(),
                    specification_set: specification_set.as_str().to_string(),
                    world_set: world_set.as_str().to_string(),
                    requirement_set: requirement_set.as_str().to_string(),
                    specification_ref: Reference {
                        name: specification_set.as_str().to_string(),
                        span: pair_to_span(&specification_set),
                    },
                    world_ref: Reference {
                        name: world_set.as_str().to_string(),
                        span: pair_to_span(&world_set),
                    },
                    requirement_ref: Reference {
                        name: requirement_set.as_str().to_string(),
                        span: pair_to_span(&requirement_set),
                    },
                    span,
                    source_path: None,
                });
            }
            _ => {}
        }
    }

    Ok(problem)
}
