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
}
