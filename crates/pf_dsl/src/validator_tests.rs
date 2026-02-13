#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::validator::{validate, ValidationError};

    fn mock_span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn mock_ref(name: &str) -> Reference {
        Reference {
            name: name.to_string(),
            span: mock_span(),
        }
    }

    fn domain(name: &str, kind: DomainKind, role: DomainRole) -> Domain {
        Domain {
            name: name.to_string(),
            kind,
            role,
            span: mock_span(),
            source_path: None,
        }
    }

    fn phenomenon(
        name: &str,
        type_: PhenomenonType,
        from: &str,
        to: &str,
        controlled_by: &str,
    ) -> Phenomenon {
        Phenomenon {
            name: name.to_string(),
            type_,
            from: mock_ref(from),
            to: mock_ref(to),
            controlled_by: mock_ref(controlled_by),
            span: mock_span(),
        }
    }

    fn interface(name: &str, connects: &[&str], shared_phenomena: Vec<Phenomenon>) -> Interface {
        Interface {
            name: name.to_string(),
            connects: connects.iter().map(|name| mock_ref(name)).collect(),
            shared_phenomena,
            span: mock_span(),
            source_path: None,
        }
    }

    #[test]
    fn test_duplicate_domain_detection() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("D1", DomainKind::Causal, DomainRole::Given),
                domain("D1", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateDomain(n, _) if n == "D1")));
    }

    #[test]
    fn test_missing_connection_commanded() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("Op", DomainKind::Biddable, DomainRole::Given),
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("C", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::CommandedBehavior,
                constrains: Some(mock_ref("C")),
                reference: Some(mock_ref("Op")),
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingConnection(d1, _, _, _) if d1 == "Op")));
    }

    #[test]
    fn test_missing_connection_required() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("C", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::RequiredBehavior,
                constrains: Some(mock_ref("C")),
                reference: None,
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::MissingConnection(d1, _, _, _) if d1 == "C")));
    }

    #[test]
    fn test_invalid_causality() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("L", DomainKind::Lexical, DomainRole::Given),
                domain("M", DomainKind::Causal, DomainRole::Machine),
            ],
            interfaces: vec![interface(
                "I1",
                &["L", "M"],
                vec![phenomenon("E1", PhenomenonType::Event, "L", "M", "L")],
            )],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(
            |e| matches!(e, ValidationError::InvalidCausality(p, _, d, _, _) if p == "E1" && d == "L")
        ));
    }

    #[test]
    fn test_invalid_command_origin() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("C", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![interface(
                "M-C",
                &["M", "C"],
                vec![phenomenon("Cmd1", PhenomenonType::Command, "M", "C", "M")],
            )],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::InvalidCausality(p, t, d, _, _)
                    if p == "Cmd1" && matches!(t, PhenomenonType::Command) && d == "M"
            )
        }));
    }

    #[test]
    fn test_missing_frame_field() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![domain("M", DomainKind::Causal, DomainRole::Machine)],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::Custom("".to_string()),
                constrains: None,
                reference: None,
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::MissingRequiredField(req, field, _)
                    if req == "R1" && field == "frame"
            )
        }));
    }

    #[test]
    fn test_unsupported_frame() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![domain("M", DomainKind::Causal, DomainRole::Machine)],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::Custom("FutureFrame".to_string()),
                constrains: None,
                reference: None,
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::UnsupportedFrame(req, frame, _)
                    if req == "R1" && frame == "FutureFrame"
            )
        }));
    }

    #[test]
    fn test_missing_reference_for_commanded_behavior() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("C", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::CommandedBehavior,
                constrains: Some(mock_ref("C")),
                reference: None,
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::MissingRequiredField(req, field, _)
                    if req == "R1" && field == "reference"
            )
        }));
    }

    #[test]
    fn test_requirement_cannot_reference_machine() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("C", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::RequiredBehavior,
                constrains: Some(mock_ref("C")),
                reference: Some(mock_ref("M")),
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::RequirementReferencesMachine(req, domain, _)
                    if req == "R1" && domain == "M"
            )
        }));
    }

    #[test]
    fn test_information_display_reference_must_be_biddable() {
        let problem = Problem {
            name: "InfoDisplay".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("Ops", DomainKind::Causal, DomainRole::Given),
                domain("Metrics", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![
                interface(
                    "Ops-M",
                    &["Ops", "M"],
                    vec![phenomenon("Push", PhenomenonType::Event, "Ops", "M", "Ops")],
                ),
                interface(
                    "Metrics-M",
                    &["Metrics", "M"],
                    vec![phenomenon(
                        "Snapshot",
                        PhenomenonType::Value,
                        "Metrics",
                        "M",
                        "Metrics",
                    )],
                ),
            ],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::InformationDisplay,
                constrains: Some(mock_ref("Metrics")),
                reference: Some(mock_ref("Ops")),
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::InvalidFrameDomain(req, frame, _, _)
                    if req == "R1" && frame == "InformationDisplay"
            )
        }));
    }

    #[test]
    fn test_simple_workpieces_requires_lexical_constrained_domain() {
        let problem = Problem {
            name: "SimpleWorkpieces".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("User", DomainKind::Biddable, DomainRole::Given),
                domain("Work", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![
                interface(
                    "User-M",
                    &["User", "M"],
                    vec![phenomenon(
                        "Edit",
                        PhenomenonType::Event,
                        "User",
                        "M",
                        "User",
                    )],
                ),
                interface(
                    "Work-M",
                    &["Work", "M"],
                    vec![phenomenon(
                        "State",
                        PhenomenonType::State,
                        "Work",
                        "M",
                        "Work",
                    )],
                ),
            ],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::SimpleWorkpieces,
                constrains: Some(mock_ref("Work")),
                reference: Some(mock_ref("User")),
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::InvalidFrameDomain(req, frame, _, _)
                    if req == "R1" && frame == "SimpleWorkpieces"
            )
        }));
    }

    #[test]
    fn test_transformation_requires_connection_to_machine() {
        let problem = Problem {
            name: "Transformation".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("Out", DomainKind::Lexical, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::Transformation,
                constrains: Some(mock_ref("Out")),
                reference: None,
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
                source_path: None,
            }],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| {
            matches!(
                e,
                ValidationError::MissingConnection(domain, _, frame, _)
                    if domain == "Out" && frame == "Transformation"
            )
        }));
    }

    #[test]
    fn test_additional_frames_valid_when_fit() {
        let problem = Problem {
            name: "AllFrames".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                domain("M", DomainKind::Causal, DomainRole::Machine),
                domain("Viewer", DomainKind::Biddable, DomainRole::Given),
                domain("Sensor", DomainKind::Causal, DomainRole::Given),
                domain("Workpiece", DomainKind::Lexical, DomainRole::Given),
                domain("Output", DomainKind::Lexical, DomainRole::Given),
            ],
            interfaces: vec![
                interface(
                    "Viewer-M",
                    &["Viewer", "M"],
                    vec![phenomenon(
                        "RequestView",
                        PhenomenonType::Event,
                        "Viewer",
                        "M",
                        "Viewer",
                    )],
                ),
                interface(
                    "Sensor-M",
                    &["Sensor", "M"],
                    vec![phenomenon(
                        "Signal",
                        PhenomenonType::Value,
                        "Sensor",
                        "M",
                        "Sensor",
                    )],
                ),
                interface(
                    "Workpiece-M",
                    &["Workpiece", "M"],
                    vec![phenomenon(
                        "Draft",
                        PhenomenonType::Value,
                        "Workpiece",
                        "M",
                        "Workpiece",
                    )],
                ),
                interface(
                    "M-Output",
                    &["M", "Output"],
                    vec![phenomenon(
                        "Produced",
                        PhenomenonType::Value,
                        "M",
                        "Output",
                        "M",
                    )],
                ),
            ],
            requirements: vec![
                Requirement {
                    name: "ShowState".to_string(),
                    frame: FrameType::InformationDisplay,
                    constrains: Some(mock_ref("Sensor")),
                    reference: Some(mock_ref("Viewer")),
                    constraint: "".to_string(),
                    phenomena: vec![],
                    span: mock_span(),
                    source_path: None,
                },
                Requirement {
                    name: "EditWorkpiece".to_string(),
                    frame: FrameType::SimpleWorkpieces,
                    constrains: Some(mock_ref("Workpiece")),
                    reference: Some(mock_ref("Viewer")),
                    constraint: "".to_string(),
                    phenomena: vec![],
                    span: mock_span(),
                    source_path: None,
                },
                Requirement {
                    name: "GenerateOutput".to_string(),
                    frame: FrameType::Transformation,
                    constrains: Some(mock_ref("Output")),
                    reference: None,
                    constraint: "".to_string(),
                    phenomena: vec![],
                    span: mock_span(),
                    source_path: None,
                },
            ],
        };

        let result = validate(&problem);
        assert!(result.is_ok());
    }
}
