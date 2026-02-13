use pf_dsl::decomposition_closure::{
    analyze_decomposition_closure, generate_decomposition_closure_markdown,
};
use pf_dsl::resolver;
use pf_dsl::validator::validate;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("decomposition")
        .join(name)
}

#[test]
fn fully_covered_fixture_has_no_decomposition_gaps() {
    let problem = resolver::resolve(&fixture_path("fully_covered.pf"), None)
        .expect("fixture must parse and resolve");
    validate(&problem).expect("fixture must pass validation");

    let closure = analyze_decomposition_closure(&problem);
    assert!(closure.uncovered_requirements.is_empty());
    assert!(closure.orphan_subproblems.is_empty());
    assert!(closure.boundary_mismatches.is_empty());

    let markdown = generate_decomposition_closure_markdown(&problem);
    assert!(markdown.contains("# Decomposition Closure Report: FullyCovered"));
    assert!(markdown.contains("### Uncovered Requirements"));
    assert!(markdown.contains("- None."));
}

#[test]
fn uncovered_fixture_flags_expected_requirement() {
    let problem = resolver::resolve(&fixture_path("intentionally_uncovered.pf"), None)
        .expect("fixture must parse and resolve");
    validate(&problem).expect("fixture must pass validation");

    let closure = analyze_decomposition_closure(&problem);
    assert_eq!(
        closure.uncovered_requirements,
        vec!["R_uncovered".to_string()]
    );
    assert!(closure.orphan_subproblems.is_empty());
    assert!(closure.boundary_mismatches.is_empty());

    let markdown = generate_decomposition_closure_markdown(&problem);
    assert!(markdown.contains("| R_uncovered | - | uncovered |"));
    assert!(markdown.contains("### Boundary Mismatches"));
}
