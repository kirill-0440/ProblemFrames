use crate::ast::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementCoverage {
    pub requirement: String,
    pub covered_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryMismatch {
    pub subproblem: String,
    pub requirement: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompositionClosure {
    pub requirement_coverage: Vec<RequirementCoverage>,
    pub uncovered_requirements: Vec<String>,
    pub orphan_subproblems: Vec<String>,
    pub boundary_mismatches: Vec<BoundaryMismatch>,
}

pub fn analyze_decomposition_closure(problem: &Problem) -> DecompositionClosure {
    let mut requirement_catalog: BTreeMap<String, &Requirement> = BTreeMap::new();
    let mut requirement_to_subproblems: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut orphan_subproblems: BTreeSet<String> = BTreeSet::new();
    let mut boundary_mismatches = Vec::new();

    for requirement in &problem.requirements {
        requirement_catalog.insert(requirement.name.clone(), requirement);
        requirement_to_subproblems.insert(requirement.name.clone(), BTreeSet::new());
    }

    for subproblem in &problem.subproblems {
        let participant_names: BTreeSet<&str> = subproblem
            .participants
            .iter()
            .map(|participant| participant.name.as_str())
            .collect();
        let mut links_known_requirement = false;

        match &subproblem.machine {
            Some(machine) => {
                if !participant_names.contains(machine.name.as_str()) {
                    boundary_mismatches.push(BoundaryMismatch {
                        subproblem: subproblem.name.clone(),
                        requirement: None,
                        detail: format!("machine '{}' is not listed in participants", machine.name),
                    });
                }
            }
            None => {
                boundary_mismatches.push(BoundaryMismatch {
                    subproblem: subproblem.name.clone(),
                    requirement: None,
                    detail: "machine is not declared".to_string(),
                });
            }
        }

        if subproblem.requirements.is_empty() {
            orphan_subproblems.insert(subproblem.name.clone());
            boundary_mismatches.push(BoundaryMismatch {
                subproblem: subproblem.name.clone(),
                requirement: None,
                detail: "subproblem does not include any requirements".to_string(),
            });
        }

        for requirement_ref in &subproblem.requirements {
            let Some(requirement) = requirement_catalog.get(requirement_ref.name.as_str()) else {
                boundary_mismatches.push(BoundaryMismatch {
                    subproblem: subproblem.name.clone(),
                    requirement: Some(requirement_ref.name.clone()),
                    detail: "subproblem references an undefined requirement".to_string(),
                });
                continue;
            };

            links_known_requirement = true;
            if let Some(covered_by) =
                requirement_to_subproblems.get_mut(requirement_ref.name.as_str())
            {
                covered_by.insert(subproblem.name.clone());
            }

            if let Some(constrains) = &requirement.constrains {
                if !participant_names.contains(constrains.name.as_str()) {
                    boundary_mismatches.push(BoundaryMismatch {
                        subproblem: subproblem.name.clone(),
                        requirement: Some(requirement.name.clone()),
                        detail: format!(
                            "missing constrained domain '{}' in participants",
                            constrains.name
                        ),
                    });
                }
            }

            if let Some(reference) = &requirement.reference {
                if !participant_names.contains(reference.name.as_str()) {
                    boundary_mismatches.push(BoundaryMismatch {
                        subproblem: subproblem.name.clone(),
                        requirement: Some(requirement.name.clone()),
                        detail: format!(
                            "missing reference domain '{}' in participants",
                            reference.name
                        ),
                    });
                }
            }
        }

        if !links_known_requirement {
            orphan_subproblems.insert(subproblem.name.clone());
        }
    }

    let mut requirement_coverage = Vec::new();
    let mut uncovered_requirements = Vec::new();

    for (requirement, covered_by) in requirement_to_subproblems {
        let covered_by = covered_by.into_iter().collect::<Vec<_>>();
        if covered_by.is_empty() {
            uncovered_requirements.push(requirement.clone());
        }
        requirement_coverage.push(RequirementCoverage {
            requirement,
            covered_by,
        });
    }

    boundary_mismatches.sort_by(|left, right| {
        (
            left.subproblem.as_str(),
            left.requirement.as_deref().unwrap_or(""),
            left.detail.as_str(),
        )
            .cmp(&(
                right.subproblem.as_str(),
                right.requirement.as_deref().unwrap_or(""),
                right.detail.as_str(),
            ))
    });

    DecompositionClosure {
        requirement_coverage,
        uncovered_requirements,
        orphan_subproblems: orphan_subproblems.into_iter().collect(),
        boundary_mismatches,
    }
}

pub fn render_decomposition_closure_section(closure: &DecompositionClosure) -> String {
    let mut output = String::new();

    output.push_str("### Requirement Coverage\n");
    if closure.requirement_coverage.is_empty() {
        output.push_str("- No requirements declared.\n\n");
    } else {
        output.push_str("| Requirement | Covered By | Status |\n");
        output.push_str("| --- | --- | --- |\n");
        for item in &closure.requirement_coverage {
            let covered_by = if item.covered_by.is_empty() {
                "-".to_string()
            } else {
                item.covered_by.join(", ")
            };
            let status = if item.covered_by.is_empty() {
                "uncovered"
            } else {
                "covered"
            };
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                item.requirement, covered_by, status
            ));
        }
        output.push('\n');
    }

    output.push_str("### Uncovered Requirements\n");
    if closure.uncovered_requirements.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for requirement in &closure.uncovered_requirements {
            output.push_str(&format!("- {}\n", requirement));
        }
        output.push('\n');
    }

    output.push_str("### Orphan Subproblems\n");
    if closure.orphan_subproblems.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for subproblem in &closure.orphan_subproblems {
            output.push_str(&format!("- {}\n", subproblem));
        }
        output.push('\n');
    }

    output.push_str("### Boundary Mismatches\n");
    if closure.boundary_mismatches.is_empty() {
        output.push_str("- None.\n");
    } else {
        for mismatch in &closure.boundary_mismatches {
            if let Some(requirement) = &mismatch.requirement {
                output.push_str(&format!(
                    "- {} / {}: {}\n",
                    mismatch.subproblem, requirement, mismatch.detail
                ));
            } else {
                output.push_str(&format!("- {}: {}\n", mismatch.subproblem, mismatch.detail));
            }
        }
    }

    output
}

pub fn generate_decomposition_closure_markdown(problem: &Problem) -> String {
    let closure = analyze_decomposition_closure(problem);
    let mut output = format!("# Decomposition Closure Report: {}\n\n", problem.name);
    output.push_str(&render_decomposition_closure_section(&closure));
    output
}

#[cfg(test)]
mod tests {
    use super::{analyze_decomposition_closure, render_decomposition_closure_section};
    use crate::ast::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn reference(name: &str) -> Reference {
        Reference {
            name: name.to_string(),
            span: span(),
        }
    }

    #[test]
    fn reports_uncovered_requirements() {
        let problem = Problem {
            name: "Closure".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![Domain {
                name: "M".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                span: span(),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![
                Requirement {
                    name: "R_covered".to_string(),
                    frame: FrameType::RequiredBehavior,
                    phenomena: vec![],
                    constraint: String::new(),
                    constrains: None,
                    reference: None,
                    span: span(),
                    source_path: None,
                },
                Requirement {
                    name: "R_uncovered".to_string(),
                    frame: FrameType::RequiredBehavior,
                    phenomena: vec![],
                    constraint: String::new(),
                    constrains: None,
                    reference: None,
                    span: span(),
                    source_path: None,
                },
            ],
            subproblems: vec![Subproblem {
                name: "S1".to_string(),
                machine: Some(reference("M")),
                participants: vec![reference("M")],
                requirements: vec![reference("R_covered")],
                span: span(),
                source_path: None,
            }],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let closure = analyze_decomposition_closure(&problem);
        assert_eq!(
            closure.uncovered_requirements,
            vec!["R_uncovered".to_string()]
        );
        assert!(closure.orphan_subproblems.is_empty());
        assert!(closure.boundary_mismatches.is_empty());

        let section = render_decomposition_closure_section(&closure);
        assert!(section.contains("| R_uncovered | - | uncovered |"));
    }
}
