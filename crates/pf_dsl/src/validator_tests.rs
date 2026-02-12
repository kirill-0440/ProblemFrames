#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::validator::{validate, ValidationError};

    fn mock_span() -> Span {
        Span { start: 0, end: 0 }
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
                },
                Domain {
                    name: "D1".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
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
                },
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                },
            ],
            interfaces: vec![], // No connection!
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::CommandedBehavior,
                constrains: "M".to_string(), // Actually Commanded constrains controlled, references Op.
                // But in our simplified model: constrains=Controlled, reference=Operator.
                // So let's align with logic: constrains="", reference="Op".
                reference: "Op".to_string(),
                constraint: "M".to_string(),
                phenomena: vec![],
                span: mock_span(),
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
                },
                Domain {
                    name: "C".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                },
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::RequiredBehavior,
                constrains: "C".to_string(),
                // In RB, Machine is implicit or just 'referenced' broadly
                reference: "".to_string(),
                constraint: "".to_string(),
                phenomena: vec![],
                span: mock_span(),
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
                },
                Domain {
                    name: "M".to_string(),
                    domain_type: DomainType::Machine,
                    span: mock_span(),
                },
            ],
            interfaces: vec![Interface {
                name: "I1".to_string(),
                shared_phenomena: vec![Phenomenon {
                    name: "E1".to_string(),
                    type_: PhenomenonType::Event,
                    from: "L".to_string(),
                    to: "M".to_string(),
                    span: mock_span(),
                }],
                span: mock_span(),
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
                },
                Domain {
                    name: "C".to_string(),
                    domain_type: DomainType::Causal,
                    span: mock_span(),
                },
            ],
            interfaces: vec![Interface {
                name: "M-C".to_string(),
                shared_phenomena: vec![Phenomenon {
                    name: "Cmd1".to_string(),
                    type_: PhenomenonType::Command,
                    from: "M".to_string(),
                    to: "C".to_string(),
                    span: mock_span(),
                }],
                span: mock_span(),
            }],
            requirements: vec![],
        };

        let result = validate(&problem);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should fail because M is Machine, not Biddable
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidCausality(p, t, d, _, _) if p == "Cmd1" && matches!(t, PhenomenonType::Command) && d == "M")));
    }
}
