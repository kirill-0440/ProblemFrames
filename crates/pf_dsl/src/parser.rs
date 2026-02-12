use crate::ast::*;
use anyhow::Result;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "problem_frames.pest"]
pub struct PFParser;

fn pair_to_span(pair: &pest::iterators::Pair<Rule>) -> Span {
    let span = pair.as_span();
    Span {
        start: span.start(),
        end: span.end(),
    }
}

pub fn parse(input: &str) -> Result<Problem> {
    let mut pairs = PFParser::parse(Rule::program, input)?;

    // Skip ROI (Start of Input) if present or just iterate
    // The top level rule is 'program', which contains (problem_decl | ...)*
    // We need to look at the inner pairs of 'program'
    let program_pair = pairs.next().unwrap();
    let problem_span = pair_to_span(&program_pair);

    let mut problem = Problem {
        name: "".to_string(),
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
                // inner: "import" ~ string_literal
                let mut inner = pair.into_inner();
                let path_literal = inner.next().unwrap().as_str();
                let path = path_literal.trim_matches('"').to_string();
                problem.imports.push(path);
            }
            Rule::problem_decl => {
                // inner: "problem:" ~ identifier
                let mut inner = pair.into_inner();
                problem.name = inner.next().unwrap().as_str().trim().to_string();
                // problem.span could be updated here if we want just the name span, but the whole file span is probably better for the root.
            }
            Rule::domain_decl => {
                // inner: "domain" ~ identifier ~ "[" ~ domain_type ~ "]"
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let type_pair = inner.next().unwrap();
                let domain_type_str = type_pair.as_str(); // e.g. "Machine"

                let domain_type = match domain_type_str {
                    "Machine" => DomainType::Machine,
                    "Causal" => DomainType::Causal,
                    "Biddable" => DomainType::Biddable,
                    "Lexical" => DomainType::Lexical,
                    "Designed" => DomainType::Designed,
                    _ => DomainType::Unknown(domain_type_str.to_string()),
                };
                problem.domains.push(Domain {
                    name,
                    domain_type,
                    span,
                    source_path: None,
                });
            }
            Rule::interface_decl => {
                // inner: "interface" ~ string_literal ~ "{" ~ shared_phenomena ~ "}"
                let mut inner = pair.into_inner();
                let name_literal = inner.next().unwrap().as_str();
                let name = name_literal.trim_matches('"').to_string();

                let shared_pair = inner.next().unwrap(); // shared_phenomena rule
                                                         // shared_phenomena inner: "shared:" ~ "{" ~ phenomenon* ~ "}"
                let mut phenomena = vec![];

                for phen_pair in shared_pair.into_inner() {
                    if phen_pair.as_rule() == Rule::phenomenon {
                        let p_span = pair_to_span(&phen_pair);
                        // inner: type ~ name ~ "[" ~ from ~ "->" ~ to ~ "]"
                        let mut p_inner = phen_pair.into_inner();
                        let type_pair = p_inner.next().unwrap();
                        let name_pair = p_inner.next().unwrap();
                        let from_pair = p_inner.next().unwrap();
                        let to_pair = p_inner.next().unwrap();

                        let type_str = type_pair.as_str();
                        let p_name = name_pair.as_str().to_string();

                        let from_ref = Reference {
                            name: from_pair.as_str().to_string(),
                            span: pair_to_span(&from_pair),
                        };
                        let to_ref = Reference {
                            name: to_pair.as_str().to_string(),
                            span: pair_to_span(&to_pair),
                        };

                        let p_type = match type_str {
                            "event" => PhenomenonType::Event,
                            "command" => PhenomenonType::Command,
                            "state" => PhenomenonType::State,
                            "value" => PhenomenonType::Value,
                            _ => PhenomenonType::Event, // default or error
                        };

                        phenomena.push(Phenomenon {
                            name: p_name,
                            type_: p_type,
                            from: from_ref,
                            to: to_ref,
                            span: p_span,
                        });
                    }
                }

                problem.interfaces.push(Interface {
                    name,
                    shared_phenomena: phenomena,
                    span,
                    source_path: None,
                });
            }
            Rule::requirement_decl => {
                // inner: "requirement" ~ string_literal ~ "{" ~ req_body ~ "}"
                let mut inner = pair.into_inner();
                let name_literal = inner.next().unwrap().as_str();
                let name = name_literal.trim_matches('"').to_string();

                let req_body_pair = inner.next().unwrap();

                let mut req = Requirement {
                    name,
                    frame: FrameType::Custom("".to_string()), // Default
                    phenomena: vec![],
                    constraint: "".to_string(),
                    constrains: None,
                    reference: None,
                    span,
                    source_path: None,
                };

                for field in req_body_pair.into_inner() {
                    match field.as_rule() {
                        Rule::frame_type => {
                            let type_str = field.into_inner().as_str();
                            req.frame = match type_str {
                                "RequiredBehavior" => FrameType::RequiredBehavior,
                                "CommandedBehavior" => FrameType::CommandedBehavior,
                                "InformationDisplay" => FrameType::InformationDisplay,
                                "SimpleWorkpieces" => FrameType::SimpleWorkpieces,
                                "Transformation" => FrameType::Transformation,
                                _ => FrameType::Custom(type_str.to_string()),
                            };
                        }
                        Rule::constraint => {
                            let s = field.into_inner().as_str();
                            req.constraint = s.trim_matches('"').to_string();
                        }
                        Rule::constrains => {
                            let ident_pair = field.into_inner().next().unwrap();
                            req.constrains = Some(Reference {
                                name: ident_pair.as_str().to_string(),
                                span: pair_to_span(&ident_pair),
                            });
                        }
                        Rule::reference => {
                            let ident_pair = field.into_inner().next().unwrap();
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
