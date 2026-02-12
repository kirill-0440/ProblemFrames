#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::parser::parse;

    #[test]
    fn test_parse_simple_problem() {
        let input = r#"
            problem: Simple
            domain D [Machine]
        "#;
        let problem = parse(input).expect("Failed to parse simple problem");
        assert_eq!(problem.name, "Simple");
        assert_eq!(problem.domains.len(), 1);
        assert_eq!(problem.domains[0].name, "D");
        assert!(matches!(
            problem.domains[0].domain_type,
            DomainType::Machine
        ));
    }

    #[test]
    fn test_parse_interface() {
        let input = r#"
            problem: I
            interface "A-B" {
                shared: {
                    event e [A -> B]
                }
            }
        "#;
        let problem = parse(input).expect("Failed to parse interface");
        assert_eq!(problem.interfaces.len(), 1);
        let iface = &problem.interfaces[0];
        assert_eq!(iface.name, "A-B");
        assert_eq!(iface.shared_phenomena.len(), 1);
        let p = &iface.shared_phenomena[0];
        assert_eq!(p.name, "e");
        assert!(matches!(p.type_, PhenomenonType::Event));
        assert_eq!(p.from, "A");
        assert_eq!(p.to, "B");
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "prob: BadKeyword"; // typo in keyword
        let result = parse(input);
        assert!(result.is_err());
    }
}
