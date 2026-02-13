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
}
