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

    set_problem_source_path(&mut problem, entry_file);

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

fn set_problem_source_path(problem: &mut Problem, source_path: &Path) {
    for domain in &mut problem.domains {
        domain.source_path = Some(source_path.to_path_buf());
    }
    for interface in &mut problem.interfaces {
        interface.source_path = Some(source_path.to_path_buf());
    }
    for requirement in &mut problem.requirements {
        requirement.source_path = Some(source_path.to_path_buf());
    }
    for subproblem in &mut problem.subproblems {
        subproblem.source_path = Some(source_path.to_path_buf());
    }
    for assertion_set in &mut problem.assertion_sets {
        assertion_set.source_path = Some(source_path.to_path_buf());
    }
    for correctness_argument in &mut problem.correctness_arguments {
        correctness_argument.source_path = Some(source_path.to_path_buf());
    }
}

fn load_standard_import(import_path_str: &str) -> Option<(&'static str, PathBuf)> {
    let content = match import_path_str {
        "std/RequiredBehavior.pf" => include_str!("../../../models/std/RequiredBehavior.pf"),
        "std/CommandedBehavior.pf" => include_str!("../../../models/std/CommandedBehavior.pf"),
        "std/InformationDisplay.pf" => include_str!("../../../models/std/InformationDisplay.pf"),
        "std/SimpleWorkpieces.pf" => include_str!("../../../models/std/SimpleWorkpieces.pf"),
        "std/Transformation.pf" => include_str!("../../../models/std/Transformation.pf"),
        _ => return None,
    };

    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("models")
        .join(import_path_str);

    Some((content, source_path))
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
        let (content, import_source_path) = if let Some((content, import_path)) =
            load_standard_import(import_path_str.as_str())
        {
            if loaded.contains(&import_path) {
                // Cycle detected or already loaded.
                continue;
            }
            loaded.insert(import_path.clone());
            (content.to_string(), import_path)
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
        for subproblem in &mut imported_problem.subproblems {
            subproblem.source_path = Some(import_source_path.clone());
        }
        for assertion_set in &mut imported_problem.assertion_sets {
            assertion_set.source_path = Some(import_source_path.clone());
        }
        for correctness_argument in &mut imported_problem.correctness_arguments {
            correctness_argument.source_path = Some(import_source_path.clone());
        }

        // Append domains, interfaces, requirements, assertions to the main problem
        // Note: This is a simple merge. Name collisions are not checked here (Validator handles that).
        problem.domains.extend(imported_problem.domains);
        problem.interfaces.extend(imported_problem.interfaces);
        problem.requirements.extend(imported_problem.requirements);
        problem.subproblems.extend(imported_problem.subproblems);
        problem
            .assertion_sets
            .extend(imported_problem.assertion_sets);
        problem
            .correctness_arguments
            .extend(imported_problem.correctness_arguments);

        // We effectively "flatten" the user's problem into one big struct.
    }

    Ok(())
}

pub fn find_definition(
    problem: &Problem,
    source_file: &Path,
    offset: usize,
) -> Option<(Option<PathBuf>, Span)> {
    let source_matches = |entity_source: Option<&PathBuf>| -> bool {
        entity_source
            .map(|path| path.as_path() == source_file)
            .unwrap_or(true)
    };

    // Helper to find domain definition
    let find_domain =
        |name: &str, preferred_source: Option<&PathBuf>| -> Option<(Option<PathBuf>, Span)> {
            let mut fallback: Option<(Option<PathBuf>, Span)> = None;
            for domain in &problem.domains {
                if domain.name != name {
                    continue;
                }

                if preferred_source
                    .map(|source| domain.source_path.as_ref() == Some(source))
                    .unwrap_or(false)
                {
                    return Some((domain.source_path.clone(), domain.span));
                }

                if fallback.is_none() {
                    fallback = Some((domain.source_path.clone(), domain.span));
                }
            }
            fallback
        };

    let find_requirement =
        |name: &str, preferred_source: Option<&PathBuf>| -> Option<(Option<PathBuf>, Span)> {
            let mut fallback: Option<(Option<PathBuf>, Span)> = None;
            for requirement in &problem.requirements {
                if requirement.name != name {
                    continue;
                }

                if preferred_source
                    .map(|source| requirement.source_path.as_ref() == Some(source))
                    .unwrap_or(false)
                {
                    return Some((requirement.source_path.clone(), requirement.span));
                }

                if fallback.is_none() {
                    fallback = Some((requirement.source_path.clone(), requirement.span));
                }
            }
            fallback
        };

    let find_assertion_set = |name: &str,
                              scope: AssertionScope,
                              preferred_source: Option<&PathBuf>|
     -> Option<(Option<PathBuf>, Span)> {
        let mut fallback: Option<(Option<PathBuf>, Span)> = None;
        for assertion_set in &problem.assertion_sets {
            if assertion_set.name != name || assertion_set.scope != scope {
                continue;
            }

            if preferred_source
                .map(|source| assertion_set.source_path.as_ref() == Some(source))
                .unwrap_or(false)
            {
                return Some((assertion_set.source_path.clone(), assertion_set.span));
            }

            if fallback.is_none() {
                fallback = Some((assertion_set.source_path.clone(), assertion_set.span));
            }
        }
        fallback
    };

    let is_offset_in_ref =
        |reference: &Reference| offset >= reference.span.start && offset < reference.span.end;

    // 1. Check Interfaces (Phenomena)
    for interface in problem
        .interfaces
        .iter()
        .filter(|interface| source_matches(interface.source_path.as_ref()))
    {
        let interface_source = interface.source_path.as_ref();
        for domain_ref in &interface.connects {
            if offset >= domain_ref.span.start && offset < domain_ref.span.end {
                return find_domain(&domain_ref.name, interface_source);
            }
        }
        for phen in &interface.shared_phenomena {
            if offset >= phen.from.span.start && offset < phen.from.span.end {
                return find_domain(&phen.from.name, interface_source);
            }
            if offset >= phen.to.span.start && offset < phen.to.span.end {
                return find_domain(&phen.to.name, interface_source);
            }
            if offset >= phen.controlled_by.span.start && offset < phen.controlled_by.span.end {
                return find_domain(&phen.controlled_by.name, interface_source);
            }
        }
    }

    // 2. Check Requirements
    for req in problem
        .requirements
        .iter()
        .filter(|req| source_matches(req.source_path.as_ref()))
    {
        let req_source = req.source_path.as_ref();
        if let Some(ref c) = req.constrains {
            if offset >= c.span.start && offset < c.span.end {
                return find_domain(&c.name, req_source);
            }
        }
        if let Some(ref r) = req.reference {
            if offset >= r.span.start && offset < r.span.end {
                return find_domain(&r.name, req_source);
            }
        }
    }

    // 3. Check Subproblems
    for subproblem in problem
        .subproblems
        .iter()
        .filter(|subproblem| source_matches(subproblem.source_path.as_ref()))
    {
        let subproblem_source = subproblem.source_path.as_ref();
        if let Some(machine_ref) = &subproblem.machine {
            if is_offset_in_ref(machine_ref) {
                return find_domain(&machine_ref.name, subproblem_source);
            }
        }

        for participant_ref in &subproblem.participants {
            if is_offset_in_ref(participant_ref) {
                return find_domain(&participant_ref.name, subproblem_source);
            }
        }

        for requirement_ref in &subproblem.requirements {
            if is_offset_in_ref(requirement_ref) {
                return find_requirement(&requirement_ref.name, subproblem_source);
            }
        }
    }

    // 4. Check Correctness Arguments
    for argument in problem
        .correctness_arguments
        .iter()
        .filter(|argument| source_matches(argument.source_path.as_ref()))
    {
        let argument_source = argument.source_path.as_ref();
        if is_offset_in_ref(&argument.specification_ref) {
            return find_assertion_set(
                &argument.specification_ref.name,
                AssertionScope::Specification,
                argument_source,
            );
        }
        if is_offset_in_ref(&argument.world_ref) {
            return find_assertion_set(
                &argument.world_ref.name,
                AssertionScope::WorldProperties,
                argument_source,
            );
        }
        if is_offset_in_ref(&argument.requirement_ref) {
            return find_assertion_set(
                &argument.requirement_ref.name,
                AssertionScope::RequirementAssertions,
                argument_source,
            );
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

    fn mock_assertion_set(
        name: &str,
        scope: AssertionScope,
        start: usize,
        end: usize,
    ) -> AssertionSet {
        AssertionSet {
            name: name.to_string(),
            scope,
            assertions: vec![Assertion {
                text: "assertion".to_string(),
                language: None,
                span: mock_span(start + 1, end.saturating_sub(1)),
            }],
            span: mock_span(start, end),
            source_path: None,
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
                marks: vec![],
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
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
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
                marks: vec![],
                span: mock_span(10, 20),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R".to_string(),
                frame: FrameType::RequiredBehavior,
                phenomena: vec![],
                marks: vec![],
                constraint: "".to_string(),
                constrains: Some(mock_ref("C", 80, 85)),
                reference: None,
                span: mock_span(70, 90),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        // Click on "C" in "constrains: C" (offset 82)
        let result = find_definition(&problem, Path::new("root.pf"), 82);
        assert!(result.is_some());
        let (_, span) = result.unwrap();
        assert_eq!(span.start, 10);
    }

    #[test]
    fn test_find_definition_from_imported_source_file() {
        let imported_path = PathBuf::from("/tmp/imported.pf");
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 200),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "A".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Machine,
                    marks: vec![],
                    span: mock_span(10, 20),
                    source_path: Some(imported_path.clone()),
                },
                Domain {
                    name: "B".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Given,
                    marks: vec![],
                    span: mock_span(21, 30),
                    source_path: Some(imported_path.clone()),
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
                source_path: Some(imported_path.clone()),
            }],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let root_result = find_definition(&problem, Path::new("/tmp/root.pf"), 52);
        assert!(
            root_result.is_none(),
            "references from imported files must not match offsets in a different source file"
        );

        let result = find_definition(&problem, Path::new("/tmp/imported.pf"), 52)
            .expect("definition should be resolved in imported file context");
        let (source_path, span) = result;
        assert_eq!(source_path.as_deref(), Some(Path::new("/tmp/imported.pf")));
        assert_eq!(span, mock_span(10, 20));
    }

    #[test]
    fn test_find_definition_subproblem_fields() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 200),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "M".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Machine,
                    marks: vec![],
                    span: mock_span(10, 20),
                    source_path: None,
                },
                Domain {
                    name: "A".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Given,
                    marks: vec![],
                    span: mock_span(21, 30),
                    source_path: None,
                },
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::RequiredBehavior,
                phenomena: vec![],
                marks: vec![],
                constraint: String::new(),
                constrains: Some(mock_ref("A", 50, 51)),
                reference: None,
                span: mock_span(40, 60),
                source_path: None,
            }],
            subproblems: vec![Subproblem {
                name: "Core".to_string(),
                machine: Some(mock_ref("M", 100, 101)),
                participants: vec![mock_ref("M", 110, 111), mock_ref("A", 112, 113)],
                requirements: vec![mock_ref("R1", 120, 124)],
                span: mock_span(90, 130),
                source_path: None,
            }],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let machine_def = find_definition(&problem, Path::new("root.pf"), 100).unwrap();
        assert_eq!(machine_def.1, mock_span(10, 20));

        let participant_def = find_definition(&problem, Path::new("root.pf"), 112).unwrap();
        assert_eq!(participant_def.1, mock_span(21, 30));

        let requirement_def = find_definition(&problem, Path::new("root.pf"), 121).unwrap();
        assert_eq!(requirement_def.1, mock_span(40, 60));
    }

    #[test]
    fn test_find_definition_correctness_argument_fields() {
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 240),
            imports: vec![],
            domains: vec![Domain {
                name: "M".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                marks: vec![],
                span: mock_span(10, 20),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![
                mock_assertion_set("S_control", AssertionScope::Specification, 30, 40),
                mock_assertion_set("W_base", AssertionScope::WorldProperties, 41, 51),
                mock_assertion_set("R_goal", AssertionScope::RequirementAssertions, 52, 62),
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A1".to_string(),
                specification_set: "S_control".to_string(),
                world_set: "W_base".to_string(),
                requirement_set: "R_goal".to_string(),
                specification_ref: mock_ref("S_control", 100, 109),
                world_ref: mock_ref("W_base", 114, 120),
                requirement_ref: mock_ref("R_goal", 128, 134),
                span: mock_span(90, 140),
                source_path: None,
            }],
        };

        let spec_def = find_definition(&problem, Path::new("root.pf"), 102).unwrap();
        assert_eq!(spec_def.1, mock_span(30, 40));

        let world_def = find_definition(&problem, Path::new("root.pf"), 116).unwrap();
        assert_eq!(world_def.1, mock_span(41, 51));

        let req_def = find_definition(&problem, Path::new("root.pf"), 130).unwrap();
        assert_eq!(req_def.1, mock_span(52, 62));
    }

    #[test]
    fn test_find_definition_prefers_same_source_on_name_collision() {
        let root_path = PathBuf::from("/tmp/root.pf");
        let imported_path = PathBuf::from("/tmp/imported.pf");
        let problem = Problem {
            name: "Test".to_string(),
            span: mock_span(0, 240),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "A".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Machine,
                    marks: vec![],
                    span: mock_span(10, 20),
                    source_path: Some(root_path.clone()),
                },
                Domain {
                    name: "A".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Given,
                    marks: vec![],
                    span: mock_span(40, 50),
                    source_path: Some(imported_path.clone()),
                },
                Domain {
                    name: "B".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Given,
                    marks: vec![],
                    span: mock_span(51, 60),
                    source_path: Some(imported_path.clone()),
                },
            ],
            interfaces: vec![Interface {
                name: "Imported".to_string(),
                connects: vec![mock_ref("A", 80, 81), mock_ref("B", 82, 83)],
                shared_phenomena: vec![Phenomenon {
                    name: "ev".to_string(),
                    type_: PhenomenonType::Event,
                    from: mock_ref("A", 90, 91),
                    to: mock_ref("B", 92, 93),
                    controlled_by: mock_ref("A", 94, 95),
                    span: mock_span(84, 100),
                }],
                span: mock_span(70, 110),
                source_path: Some(imported_path.clone()),
            }],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let imported_result =
            find_definition(&problem, imported_path.as_path(), 90).expect("definition expected");
        assert_eq!(imported_result.0.as_deref(), Some(imported_path.as_path()));
        assert_eq!(imported_result.1, mock_span(40, 50));
    }
}
