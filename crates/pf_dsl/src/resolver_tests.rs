#[cfg(test)]
mod tests {
    use crate::resolver::resolve;
    use std::path::Path;

    #[test]
    fn test_resolve_standard_library_embedded() {
        // Test that importing std/ works even if the file doesn't exist on disk relative to a dummy path.
        // We use a dummy file as context.
        let input = r#"
            problem: EmbeddedStdTest
            import "std/RequiredBehavior.pf"
        "#;

        // We use a dummy path "test.pf" but pass content manually.
        // The resolver should see "std/..." and use the embedded version.
        // However, resolve() takes a path to resolve relative imports.
        let dummy_path = Path::new("dummy.pf");

        let problem = resolve(dummy_path, Some(input)).expect("Failed to resolve with std import");

        // Check if Machine domain from std lib is present
        assert!(problem.domains.iter().any(|d| d.name == "Machine"));
        assert!(problem.domains.iter().any(|d| d.name == "ControlledDomain"));
    }

    #[test]
    fn test_resolve_extended_standard_library_embedded() {
        let input = r#"
            problem: ExtendedStdTest
            import "std/InformationDisplay.pf"
            import "std/SimpleWorkpieces.pf"
            import "std/Transformation.pf"
        "#;

        let dummy_path = Path::new("dummy.pf");
        let problem = resolve(dummy_path, Some(input)).expect("Failed to resolve with std imports");

        assert!(problem
            .requirements
            .iter()
            .any(|r| r.name == "InformationDisplay"));
        assert!(problem
            .requirements
            .iter()
            .any(|r| r.name == "SimpleWorkpieces"));
        assert!(problem
            .requirements
            .iter()
            .any(|r| r.name == "Transformation"));
    }

    #[test]
    fn test_resolve_unknown_standard_file() {
        let input = r#"
            problem: Fail
            import "std/Unknown.pf"
        "#;
        let dummy_path = Path::new("dummy.pf");
        let result = resolve(dummy_path, Some(input));
        assert!(result.is_err());
    }
}
