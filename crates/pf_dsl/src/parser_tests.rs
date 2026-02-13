#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::parse;

    #[test]
    fn test_parse_simple_problem() {
        let input = r#"
            problem: Simple
            domain D kind causal role machine
        "#;
        let problem = parse(input).expect("Failed to parse simple problem");
        assert_eq!(problem.name, "Simple");
        assert_eq!(problem.domains.len(), 1);
        assert_eq!(problem.domains[0].name, "D");
        assert!(matches!(problem.domains[0].kind, DomainKind::Causal));
        assert!(matches!(problem.domains[0].role, DomainRole::Machine));
    }

    #[test]
    fn test_parse_interface() {
        let input = r#"
            problem: I
            interface "A-B" connects A, B {
                shared: {
                    phenomenon e : event [A -> B] controlledBy A
                }
            }
        "#;
        let problem = parse(input).expect("Failed to parse interface");
        assert_eq!(problem.interfaces.len(), 1);
        let iface = &problem.interfaces[0];
        assert_eq!(iface.name, "A-B");
        assert_eq!(iface.connects.len(), 2);
        assert_eq!(iface.shared_phenomena.len(), 1);
        let p = &iface.shared_phenomena[0];
        assert_eq!(p.name, "e");
        assert!(matches!(p.type_, PhenomenonType::Event));
        assert_eq!(p.from.name, "A");
        assert_eq!(p.to.name, "B");
        assert_eq!(p.controlled_by.name, "A");
    }

    #[test]
    fn test_parse_new_types() {
        let input = r#"
            problem: NewTypes
            domain Des kind causal role designed
            interface "Des-Mac" connects Des, Mac {
                shared: {
                    phenomenon Cmd1 : command [Des->Mac] controlledBy Des
                }
            }
        "#;
        let problem = parse(input).expect("Failed to parse");
        assert_eq!(problem.domains[0].kind, DomainKind::Causal);
        assert_eq!(problem.domains[0].role, DomainRole::Designed);
        assert!(matches!(
            problem.interfaces[0].shared_phenomena[0].type_,
            PhenomenonType::Command
        ));
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "prob: BadKeyword"; // typo in keyword
        let result = parse(input);
        assert!(result.is_err());
    }
}
