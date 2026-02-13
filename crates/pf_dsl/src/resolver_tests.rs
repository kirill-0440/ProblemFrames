#[cfg(test)]
mod tests {
    use crate::resolver::resolve;
    use crate::validator::{validate_with_sources, ValidationError};
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

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

    #[test]
    fn test_import_name_collisions_are_validation_errors_for_all_top_level_entities() {
        let dir = make_temp_dir("pf-import-collision-policy");
        let root_path = dir.join("root.pf");
        let a_path = dir.join("a.pf");
        let b_path = dir.join("b.pf");

        let module = r#"
problem: Imported
domain M kind causal role machine
domain D kind causal role given
interface "I" connects M, D {
  shared: {
    phenomenon E : event [M -> D] controlledBy M
  }
}
requirement "R" {
  frame: RequiredBehavior
  constrains: D
}
subproblem S {
  machine: M
  participants: M, D
  requirements: "R"
}
worldProperties W {
  assert "world"
}
specification S_spec {
  assert "spec"
}
requirementAssertions R_set {
  assert "req"
}
correctnessArgument A {
  prove S_spec and W entail R_set
}
"#;

        fs::write(&a_path, module).expect("failed to write first import");
        fs::write(&b_path, module).expect("failed to write second import");
        fs::write(
            &root_path,
            "problem: Root\nimport \"a.pf\"\nimport \"b.pf\"\n",
        )
        .expect("failed to write root");

        let problem = resolve(&root_path, None).expect("failed to resolve imports");
        let issues =
            validate_with_sources(&problem).expect_err("expected duplicate validation errors");

        let has_error_for_b = |predicate: &dyn Fn(&ValidationError) -> bool| {
            issues.iter().any(|issue| {
                predicate(&issue.error)
                    && issue
                        .source_path
                        .as_ref()
                        .and_then(|path| path.file_name())
                        .map(|name| name == "b.pf")
                        .unwrap_or(false)
            })
        };

        assert!(has_error_for_b(&|error| {
            matches!(error, ValidationError::DuplicateDomain(_, _, _))
        }));
        assert!(has_error_for_b(&|error| {
            matches!(error, ValidationError::DuplicateInterface(_, _, _))
        }));
        assert!(has_error_for_b(&|error| {
            matches!(error, ValidationError::DuplicateRequirement(_, _, _))
        }));
        assert!(has_error_for_b(&|error| {
            matches!(error, ValidationError::DuplicateSubproblem(_, _, _))
        }));
        assert!(has_error_for_b(&|error| {
            matches!(error, ValidationError::DuplicateAssertionSet(_, _, _))
        }));
        assert!(has_error_for_b(&|error| {
            matches!(
                error,
                ValidationError::DuplicateCorrectnessArgument(_, _, _)
            )
        }));

        let _ = fs::remove_dir_all(dir);
    }
}
