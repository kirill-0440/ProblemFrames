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
        // Append domains, interfaces, requirements to the main problem
        // Note: This is a simple merge. Name collisions are not checked here (Validator handles that).
        problem.domains.extend(imported_problem.domains);
        problem.interfaces.extend(imported_problem.interfaces);
        problem.requirements.extend(imported_problem.requirements);

        // We effectively "flatten" the user's problem into one big struct.
    }

    Ok(())
}
