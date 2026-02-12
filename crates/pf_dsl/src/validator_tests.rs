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

    #[test]
    fn test_duplicate_domain_detection() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "D1".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "D1".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                    source_path: None,
                },
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
        // Operator is not connected to any Machine
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "Op".to_string(),
                    domain_type: DomainType::Biddable,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
            ],
            interfaces: vec![], // No connection!
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::CommandedBehavior,
                constrains: None,
                reference: Some(mock_ref("Op")),
                constraint: "M".to_string(),
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
        // Machine is not connected to Controlled Domain
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "C".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                    source_path: None,
                },
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
        // Event originating from Lexical domain
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "L".to_string(),
                    domain_type: DomainType::Lexical,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
            ],
            interfaces: vec![Interface {
                name: "I1".to_string(),
                shared_phenomena: vec![Phenomenon {
                    name: "E1".to_string(),
                    type_: PhenomenonType::Event,
                    from: mock_ref("L"),
                    to: mock_ref("M"),
                    span: mock_span(),
                }],
                span: mock_span(),
                source_path: None,
            }],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Check for InvalidCausality
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidCausality(p, _, d, _, _) if p == "E1" && d == "L")));
    }

    #[test]
    fn test_invalid_command_origin() {
        // Command originating from Machine (should be Operator/Biddable)
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "C".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                    source_path: None,
                },
            ],
            interfaces: vec![Interface {
                name: "M-C".to_string(),
                shared_phenomena: vec![Phenomenon {
                    name: "Cmd1".to_string(),
                    type_: PhenomenonType::Command,
                    from: mock_ref("M"),
                    to: mock_ref("C"),
                    span: mock_span(),
                }],
                span: mock_span(),
                source_path: None,
            }],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should fail because M is Machine, not Biddable
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidCausality(p, t, d, _, _) if p == "Cmd1" && matches!(t, PhenomenonType::Command) && d == "M")));
    }

    #[test]
    fn test_missing_frame_field() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(),
            imports: vec![],
            domains: vec![],
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
            domains: vec![],
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
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                    source_path: None,
                },
                Domain {
                    name: "C".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                    source_path: None,
                },
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
}
