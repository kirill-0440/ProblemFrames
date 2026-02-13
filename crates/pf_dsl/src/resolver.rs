use crate::ast::*;
use crate::parser::parse;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve(entry_file: &Path, content_override: Option<&str>) -> Result<Problem> {
    let content = match content_override {
        Some(c) => c.to_string(),
        None => fs::read_to_string(entry_file)
            .with_context(|| format!("Failed to read file: {:?}", entry_file))?,
    };

    let mut problem =
        parse(&content).with_context(|| format!("Failed to parse file: {:?}", entry_file))?;

    // Track loaded files to avoid cycles (simple check)
    let mut loaded = HashSet::new();
    if let Ok(canon) = fs::canonicalize(entry_file) {
        loaded.insert(canon);
    } else {
        // If file doesn't exist on disk (e.g. unsaved buffer), we can't canonicalize it easily.
        // We just use the path as is for now, or skip adding it to loaded.
        // For imports to work relative to it, it must have a parent.
    }

    resolve_recursive(&mut problem, entry_file, &mut loaded)?;

    Ok(problem)
}

fn resolve_recursive(
    problem: &mut Problem,
    current_file: &Path,
    loaded: &mut HashSet<PathBuf>,
) -> Result<()> {
    let base_dir = current_file.parent().unwrap_or(Path::new("."));

    // We iterate over a copy of imports to avoid borrow issues
    let imports = problem.imports.clone();

    for import_path_str in imports {
        let (content, import_source_path) = if import_path_str.starts_with("std/") {
            // Handle Standard Library
            let content = match import_path_str.as_str() {
                "std/RequiredBehavior.pf" => include_str!("std/RequiredBehavior.pf").to_string(),
                "std/CommandedBehavior.pf" => include_str!("std/CommandedBehavior.pf").to_string(),
                "std/InformationDisplay.pf" => {
                    include_str!("std/InformationDisplay.pf").to_string()
                }
                "std/SimpleWorkpieces.pf" => include_str!("std/SimpleWorkpieces.pf").to_string(),
                "std/Transformation.pf" => include_str!("std/Transformation.pf").to_string(),
                _ => anyhow::bail!("Standard library file not found: {}", import_path_str),
            };
            // For std imports, we use the import string itself as a unique identifier
            // and a dummy path for recursion context.
            (content, PathBuf::from(import_path_str.clone()))
        } else {
            let import_path = base_dir.join(&import_path_str);

            let canonical_path = fs::canonicalize(&import_path)
                .with_context(|| format!("Failed to resolve import path: {:?}", import_path))?;

            if loaded.contains(&canonical_path) {
                // Cycle detected or already loaded. For v1 we just skip (idempotent include)
                continue;
            }
            loaded.insert(canonical_path.clone());

            let content = fs::read_to_string(&canonical_path)
                .with_context(|| format!("Failed to read imported file: {:?}", canonical_path))?;
            (content, canonical_path)
        };

        let mut imported_problem = parse(&content)
            .with_context(|| format!("Failed to parse imported file: {:?}", import_source_path))?;

        // Recursively resolve imports of the imported problem
        resolve_recursive(&mut imported_problem, &import_source_path, loaded)?;

        // MERGE LOGIC:
        // Set source_path for all imported elements
        for domain in &mut imported_problem.domains {
            domain.source_path = Some(import_source_path.clone());
        }
        for interface in &mut imported_problem.interfaces {
            interface.source_path = Some(import_source_path.clone());
        }
        for requirement in &mut imported_problem.requirements {
            requirement.source_path = Some(import_source_path.clone());
        }

        // Append domains, interfaces, requirements to the main problem
        // Note: This is a simple merge. Name collisions are not checked here (Validator handles that).
        problem.domains.extend(imported_problem.domains);
        problem.interfaces.extend(imported_problem.interfaces);
        problem.requirements.extend(imported_problem.requirements);

        // We effectively "flatten" the user's problem into one big struct.
    }

    Ok(())
}

fn path_eq(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

pub fn find_definition(
    problem: &Problem,
    source_file: &Path,
    offset: usize,
) -> Option<(Option<PathBuf>, Span)> {
    // Helper to find domain definition
    let find_domain = |name: &str| -> Option<(Option<PathBuf>, Span)> {
        problem
            .domains
            .iter()
            .find(|d| d.name == name)
            .map(|d| (d.source_path.clone(), d.span))
    };

    // 1. Check Interfaces (Phenomena)
    for interface in &problem.interfaces {
        if let Some(source_path) = interface.source_path.as_deref() {
            if !path_eq(source_path, source_file) {
                continue;
            }
        }

        for domain_ref in &interface.connects {
            if offset >= domain_ref.span.start && offset < domain_ref.span.end {
                return find_domain(&domain_ref.name);
            }
        }
        for phen in &interface.shared_phenomena {
            if offset >= phen.from.span.start && offset < phen.from.span.end {
                return find_domain(&phen.from.name);
            }
            if offset >= phen.to.span.start && offset < phen.to.span.end {
                return find_domain(&phen.to.name);
            }
            if offset >= phen.controlled_by.span.start && offset < phen.controlled_by.span.end {
                return find_domain(&phen.controlled_by.name);
            }
        }
    }

    // 2. Check Requirements
    for req in &problem.requirements {
        if let Some(source_path) = req.source_path.as_deref() {
            if !path_eq(source_path, source_file) {
                continue;
            }
        }

        if let Some(ref c) = req.constrains {
            if offset >= c.span.start && offset < c.span.end {
                return find_domain(&c.name);
            }
        }
        if let Some(ref r) = req.reference {
            if offset >= r.span.start && offset < r.span.end {
                return find_domain(&r.name);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_span(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    fn mock_ref(name: &str, start: usize, end: usize) -> Reference {
        Reference {
            name: name.to_string(),
            span: mock_span(start, end),
        }
    }

    #[test]
    fn test_find_definition_phenomenon() {
        // Domain D defined at 10..20
        // Phenomenon E from D (referenced at 50..55)
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 100),
            imports: vec![],
            domains: vec![Domain {
                name: "D".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                span: mock_span(10, 20),
                source_path: None,
            }],
            interfaces: vec![Interface {
                name: "I".to_string(),
                connects: vec![mock_ref("D", 32, 33), mock_ref("X", 34, 35)],
                shared_phenomena: vec![Phenomenon {
                    name: "E".to_string(),
                    type_: PhenomenonType::Event,
                    from: mock_ref("D", 50, 55),
                    to: mock_ref("X", 60, 65), // X not defined
                    controlled_by: mock_ref("D", 66, 71),
                    span: mock_span(40, 70),
                }],
                span: mock_span(30, 80),
                source_path: None,
            }],
            requirements: vec![],
        };

        // Click on "D" in "from D" (offset 52)
        let result = find_definition(&problem, Path::new("root.pf"), 52);
        assert!(result.is_some());
        let (_, span) = result.unwrap();
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);

        // Click on "X" (offset 62) -> should be None as X is not in domains
        let result = find_definition(&problem, Path::new("root.pf"), 62);
        assert!(result.is_none());

        // Click nowhere (offset 0)
        let result = find_definition(&problem, Path::new("root.pf"), 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_definition_requirement() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 100),
            imports: vec![],
            domains: vec![Domain {
                name: "C".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Given,
                span: mock_span(10, 20),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R".to_string(),
                frame: FrameType::RequiredBehavior,
                phenomena: vec![],
                constraint: "".to_string(),
                constrains: Some(mock_ref("C", 80, 85)),
                reference: None,
                span: mock_span(70, 90),
                source_path: None,
            }],
        };

        // Click on "C" in "constrains: C" (offset 82)
        let result = find_definition(&problem, Path::new("root.pf"), 82);
        assert!(result.is_some());
        let (_, span) = result.unwrap();
        assert_eq!(span.start, 10);
    }

    #[test]
    fn test_find_definition_ignores_other_source_files() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 200),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "A".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Machine,
                    span: mock_span(10, 20),
                    source_path: None,
                },
                Domain {
                    name: "B".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Given,
                    span: mock_span(21, 30),
                    source_path: None,
                },
            ],
            interfaces: vec![Interface {
                name: "Imported".to_string(),
                connects: vec![mock_ref("A", 40, 41), mock_ref("B", 42, 43)],
                shared_phenomena: vec![Phenomenon {
                    name: "ev".to_string(),
                    type_: PhenomenonType::Event,
                    from: mock_ref("A", 50, 55),
                    to: mock_ref("B", 56, 61),
                    controlled_by: mock_ref("A", 62, 67),
                    span: mock_span(44, 70),
                }],
                span: mock_span(30, 80),
                source_path: Some(PathBuf::from("/tmp/imported.pf")),
            }],
            requirements: vec![],
        };

        let result = find_definition(&problem, Path::new("/tmp/root.pf"), 52);
        assert!(result.is_none());
    }
}
