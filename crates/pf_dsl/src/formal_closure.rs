use crate::ast::{FrameType, Problem, Requirement};

pub const FORMAL_ARGUMENT_MARK: &str = "formal.argument";

fn requirement_formal_argument(requirement: &Requirement) -> Option<String> {
    requirement
        .marks
        .iter()
        .find(|mark| mark.name == FORMAL_ARGUMENT_MARK)
        .and_then(|mark| mark.value.as_ref())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

pub fn formal_closure_rows(problem: &Problem) -> Vec<(String, String)> {
    let mut rows = problem
        .requirements
        .iter()
        .filter_map(|requirement| {
            requirement_formal_argument(requirement)
                .map(|argument| (requirement.name.clone(), argument))
        })
        .collect::<Vec<_>>();

    rows.sort_by(|left, right| left.0.cmp(&right.0));
    rows
}

pub fn generate_formal_closure_map_tsv(problem: &Problem) -> String {
    let mut output = String::from("# requirement|correctness_argument\n");
    for (requirement, argument) in formal_closure_rows(problem) {
        output.push_str(&format!("{}|{}\n", requirement, argument));
    }
    output
}

fn frame_name(frame: &FrameType) -> String {
    match frame {
        FrameType::RequiredBehavior => "RequiredBehavior".to_string(),
        FrameType::CommandedBehavior => "CommandedBehavior".to_string(),
        FrameType::InformationDisplay => "InformationDisplay".to_string(),
        FrameType::SimpleWorkpieces => "SimpleWorkpieces".to_string(),
        FrameType::Transformation => "Transformation".to_string(),
        FrameType::Custom(name) => name.clone(),
    }
}

pub fn generate_requirements_tsv(problem: &Problem) -> String {
    let mut rows = problem
        .requirements
        .iter()
        .map(|requirement| (requirement.name.clone(), frame_name(&requirement.frame)))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0));

    let mut output = String::from("# requirement|frame\n");
    for (requirement, frame) in rows {
        output.push_str(&format!("{}|{}\n", requirement, frame));
    }
    output
}

pub fn generate_correctness_arguments_tsv(problem: &Problem) -> String {
    let mut rows = problem
        .correctness_arguments
        .iter()
        .map(|argument| argument.name.clone())
        .collect::<Vec<_>>();
    rows.sort();

    let mut output = String::from("# correctness_argument\n");
    for argument in rows {
        output.push_str(&format!("{}\n", argument));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        formal_closure_rows, generate_correctness_arguments_tsv, generate_formal_closure_map_tsv,
        generate_requirements_tsv,
    };
    use crate::ast::{
        Domain, DomainKind, DomainRole, FrameType, Mark, Problem, Reference, Requirement, Span,
    };

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn requirement(name: &str, mark_value: Option<&str>) -> Requirement {
        Requirement {
            name: name.to_string(),
            frame: FrameType::SimpleWorkpieces,
            phenomena: vec![],
            marks: mark_value
                .map(|value| {
                    vec![Mark {
                        name: "formal.argument".to_string(),
                        value: Some(value.to_string()),
                        span: span(),
                    }]
                })
                .unwrap_or_default(),
            constraint: "x".to_string(),
            constrains: Some(Reference {
                name: "Tool".to_string(),
                span: span(),
            }),
            reference: None,
            span: span(),
            source_path: None,
        }
    }

    #[test]
    fn generates_tsv_from_requirement_marks() {
        let problem = Problem {
            name: "FormalClosure".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![Domain {
                name: "Tool".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                marks: vec![],
                span: span(),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![
                requirement("R2", Some("A2")),
                requirement("R1", Some("A1")),
                requirement("R3", None),
            ],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        assert_eq!(
            formal_closure_rows(&problem),
            vec![
                ("R1".to_string(), "A1".to_string()),
                ("R2".to_string(), "A2".to_string()),
            ],
        );

        let tsv = generate_formal_closure_map_tsv(&problem);
        assert!(tsv.contains("# requirement|correctness_argument"));
        assert!(tsv.contains("R1|A1"));
        assert!(tsv.contains("R2|A2"));
        assert!(!tsv.contains("R3|"));
    }

    #[test]
    fn generates_requirement_and_argument_inventories() {
        let problem = Problem {
            name: "Inventory".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![Domain {
                name: "Tool".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                marks: vec![],
                span: span(),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![
                Requirement {
                    name: "R2".to_string(),
                    frame: FrameType::SimpleWorkpieces,
                    phenomena: vec![],
                    marks: vec![],
                    constraint: "x".to_string(),
                    constrains: Some(Reference {
                        name: "Tool".to_string(),
                        span: span(),
                    }),
                    reference: None,
                    span: span(),
                    source_path: None,
                },
                Requirement {
                    name: "R1".to_string(),
                    frame: FrameType::Transformation,
                    phenomena: vec![],
                    marks: vec![],
                    constraint: "y".to_string(),
                    constrains: Some(Reference {
                        name: "Tool".to_string(),
                        span: span(),
                    }),
                    reference: None,
                    span: span(),
                    source_path: None,
                },
            ],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![
                crate::ast::CorrectnessArgument {
                    name: "A2".to_string(),
                    specification_set: "S".to_string(),
                    world_set: "W".to_string(),
                    requirement_set: "R".to_string(),
                    specification_ref: Reference {
                        name: "S".to_string(),
                        span: span(),
                    },
                    world_ref: Reference {
                        name: "W".to_string(),
                        span: span(),
                    },
                    requirement_ref: Reference {
                        name: "R".to_string(),
                        span: span(),
                    },
                    span: span(),
                    source_path: None,
                },
                crate::ast::CorrectnessArgument {
                    name: "A1".to_string(),
                    specification_set: "S".to_string(),
                    world_set: "W".to_string(),
                    requirement_set: "R".to_string(),
                    specification_ref: Reference {
                        name: "S".to_string(),
                        span: span(),
                    },
                    world_ref: Reference {
                        name: "W".to_string(),
                        span: span(),
                    },
                    requirement_ref: Reference {
                        name: "R".to_string(),
                        span: span(),
                    },
                    span: span(),
                    source_path: None,
                },
            ],
        };

        let requirements_tsv = generate_requirements_tsv(&problem);
        assert!(requirements_tsv.contains("# requirement|frame"));
        assert!(requirements_tsv.contains("R1|Transformation"));
        assert!(requirements_tsv.contains("R2|SimpleWorkpieces"));

        let arguments_tsv = generate_correctness_arguments_tsv(&problem);
        assert!(arguments_tsv.contains("# correctness_argument"));
        assert!(arguments_tsv.contains("A1"));
        assert!(arguments_tsv.contains("A2"));
    }
}
