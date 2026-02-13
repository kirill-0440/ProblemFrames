#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Deserialize)]
    struct InvariantCatalog {
        catalog_version: String,
        scope: String,
        rules: Vec<InvariantRule>,
    }

    #[derive(Debug, Deserialize)]
    struct InvariantRule {
        rule_id: String,
        error_variant: String,
        severity: String,
        title: String,
        rationale: String,
        validator_paths: Vec<String>,
        valid_tests: Vec<String>,
        invalid_tests: Vec<String>,
    }

    #[derive(Debug)]
    struct MatrixRow {
        rule_id: String,
        error_variant: String,
        valid_test: String,
        invalid_test: String,
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .canonicalize()
            .expect("failed to resolve repository root")
    }

    fn read_file(relative_path: &str) -> String {
        let path = repo_root().join(relative_path);
        fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("failed to read '{}': {}", path.to_string_lossy(), error)
        })
    }

    fn load_catalog() -> InvariantCatalog {
        let raw = read_file("metamodel/invariant-catalog.json");
        serde_json::from_str(&raw).expect("invariant-catalog.json must be valid JSON")
    }

    fn parse_validation_error_variants(src: &str) -> BTreeSet<String> {
        let enum_anchor = src
            .find("pub enum ValidationError")
            .expect("ValidationError enum not found");
        let enum_body_start = src[enum_anchor..]
            .find('{')
            .map(|offset| enum_anchor + offset + 1)
            .expect("ValidationError opening brace not found");

        let mut variants = BTreeSet::new();
        for line in src[enum_body_start..].lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('}') {
                break;
            }
            if trimmed.is_empty() || trimmed.starts_with("#[") {
                continue;
            }
            if let Some((candidate, _)) = trimmed.split_once('(') {
                if candidate
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    variants.insert(candidate.to_string());
                }
            }
        }

        variants
    }

    fn parse_test_blocks(src: &str) -> BTreeMap<String, String> {
        let lines: Vec<&str> = src.lines().collect();
        let mut blocks = BTreeMap::new();
        let mut cursor = 0;

        while cursor < lines.len() {
            if lines[cursor].trim() != "#[test]" {
                cursor += 1;
                continue;
            }

            let mut signature_index = cursor + 1;
            while signature_index < lines.len()
                && !lines[signature_index].trim_start().starts_with("fn ")
            {
                signature_index += 1;
            }
            if signature_index >= lines.len() {
                break;
            }

            let signature_line = lines[signature_index].trim_start();
            let test_name = signature_line
                .strip_prefix("fn ")
                .and_then(|tail| tail.split('(').next())
                .expect("test function name parse failed")
                .trim()
                .to_string();

            let mut next_test = signature_index + 1;
            while next_test < lines.len() && lines[next_test].trim() != "#[test]" {
                next_test += 1;
            }

            let body = lines[signature_index..next_test].join("\n");
            blocks.insert(test_name, body);
            cursor = next_test;
        }

        blocks
    }

    fn load_matrix() -> Vec<MatrixRow> {
        let raw = read_file("metamodel/rule-test-matrix.tsv");
        let mut rows = Vec::new();

        for (line_no, line) in raw.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            if line_no == 0 {
                assert_eq!(
                    line, "rule_id\terror_variant\tvalid_test\tinvalid_test",
                    "rule-test-matrix.tsv header mismatch"
                );
                continue;
            }

            let columns: Vec<&str> = line.split('\t').collect();
            assert_eq!(
                columns.len(),
                4,
                "rule-test-matrix.tsv line {} must have 4 tab-separated columns",
                line_no + 1
            );

            rows.push(MatrixRow {
                rule_id: columns[0].to_string(),
                error_variant: columns[1].to_string(),
                valid_test: columns[2].to_string(),
                invalid_test: columns[3].to_string(),
            });
        }

        rows
    }

    #[test]
    fn test_metamodel_catalog_is_complete_and_well_formed() {
        let catalog = load_catalog();
        assert!(
            !catalog.catalog_version.trim().is_empty(),
            "catalog_version must be non-empty"
        );
        assert!(!catalog.scope.trim().is_empty(), "scope must be non-empty");
        assert!(
            !catalog.rules.is_empty(),
            "catalog must contain at least one rule"
        );

        let validator_source = read_file("crates/pf_dsl/src/validator.rs");
        let validator_variants = parse_validation_error_variants(&validator_source);
        let allowed_severities = ["error", "warn", "info"];

        let mut rule_ids = BTreeSet::new();
        let mut catalog_variants = BTreeSet::new();

        for rule in &catalog.rules {
            assert!(
                rule_ids.insert(rule.rule_id.clone()),
                "duplicate rule_id '{}'",
                rule.rule_id
            );
            assert!(
                catalog_variants.insert(rule.error_variant.clone()),
                "duplicate error_variant '{}' in catalog",
                rule.error_variant
            );
            assert!(
                allowed_severities.contains(&rule.severity.as_str()),
                "rule '{}' has unsupported severity '{}'",
                rule.rule_id,
                rule.severity
            );
            assert!(
                !rule.title.trim().is_empty(),
                "rule '{}' must have title",
                rule.rule_id
            );
            assert!(
                !rule.rationale.trim().is_empty(),
                "rule '{}' must have rationale",
                rule.rule_id
            );
            assert!(
                !rule.validator_paths.is_empty(),
                "rule '{}' must reference validator paths",
                rule.rule_id
            );
            assert!(
                !rule.valid_tests.is_empty(),
                "rule '{}' must list at least one valid test",
                rule.rule_id
            );
            assert!(
                !rule.invalid_tests.is_empty(),
                "rule '{}' must list at least one invalid test",
                rule.rule_id
            );
        }

        assert_eq!(
            catalog_variants, validator_variants,
            "catalog error variants must exactly match ValidationError variants"
        );
    }

    #[test]
    fn test_metamodel_catalog_test_references_are_executable() {
        let catalog = load_catalog();
        let test_source = read_file("crates/pf_dsl/src/validator_tests.rs");
        let test_blocks = parse_test_blocks(&test_source);

        for rule in &catalog.rules {
            let variant_token = format!("ValidationError::{}", rule.error_variant);

            for valid_test in &rule.valid_tests {
                let body = test_blocks.get(valid_test).unwrap_or_else(|| {
                    panic!(
                        "rule '{}' references unknown valid test '{}'",
                        rule.rule_id, valid_test
                    )
                });
                assert!(
                    body.contains("assert!(result.is_ok())"),
                    "valid test '{}' for rule '{}' must assert success",
                    valid_test,
                    rule.rule_id
                );
            }

            for invalid_test in &rule.invalid_tests {
                let body = test_blocks.get(invalid_test).unwrap_or_else(|| {
                    panic!(
                        "rule '{}' references unknown invalid test '{}'",
                        rule.rule_id, invalid_test
                    )
                });
                assert!(
                    body.contains("assert!(result.is_err())"),
                    "invalid test '{}' for rule '{}' must assert failure",
                    invalid_test,
                    rule.rule_id
                );
                assert!(
                    body.contains(&variant_token),
                    "invalid test '{}' for rule '{}' must assert '{}'",
                    invalid_test,
                    rule.rule_id,
                    variant_token
                );
            }
        }
    }

    #[test]
    fn test_metamodel_rule_test_matrix_is_synced_with_catalog() {
        let catalog = load_catalog();
        let matrix_rows = load_matrix();

        assert_eq!(
            matrix_rows.len(),
            catalog.rules.len(),
            "rule-test matrix row count must match catalog rules"
        );

        let matrix_by_rule_id: BTreeMap<String, &MatrixRow> = matrix_rows
            .iter()
            .map(|row| (row.rule_id.clone(), row))
            .collect();
        assert_eq!(
            matrix_by_rule_id.len(),
            matrix_rows.len(),
            "rule-test matrix contains duplicate rule IDs"
        );

        for rule in &catalog.rules {
            let row = matrix_by_rule_id.get(&rule.rule_id).unwrap_or_else(|| {
                panic!("missing matrix row for catalog rule '{}'", rule.rule_id)
            });
            assert_eq!(
                row.error_variant, rule.error_variant,
                "matrix variant mismatch for rule '{}'",
                rule.rule_id
            );
            assert!(
                rule.valid_tests.contains(&row.valid_test),
                "matrix valid test '{}' is not listed in catalog rule '{}'",
                row.valid_test,
                rule.rule_id
            );
            assert!(
                rule.invalid_tests.contains(&row.invalid_test),
                "matrix invalid test '{}' is not listed in catalog rule '{}'",
                row.invalid_test,
                rule.rule_id
            );
        }
    }
}
