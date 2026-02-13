use crate::ast::Problem;
use crate::wrspm;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcernCoverageState {
    Covered,
    Uncovered,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcernCoverageGateStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirementConcernCoverage {
    pub requirement: String,
    pub subproblems: Vec<String>,
    pub correctness_arguments: Vec<String>,
    pub wrspm_w_sets: Vec<String>,
    pub wrspm_s_sets: Vec<String>,
    pub wrspm_r_sets: Vec<String>,
    pub state: ConcernCoverageState,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcernCoverageSummary {
    pub total_requirements: usize,
    pub covered_requirements: usize,
    pub uncovered_requirements: Vec<String>,
    pub deferred_requirements: Vec<String>,
    pub requirement_rows: Vec<RequirementConcernCoverage>,
    pub gate_status: ConcernCoverageGateStatus,
    pub wrspm_p_status: String,
    pub wrspm_m_status: String,
    pub wrspm_unresolved: Vec<String>,
}

pub fn summarize(problem: &Problem) -> ConcernCoverageSummary {
    let wrspm_projection = wrspm::project(problem);
    let mut requirement_to_subproblems: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for subproblem in &problem.subproblems {
        for requirement in &subproblem.requirements {
            requirement_to_subproblems
                .entry(requirement.name.clone())
                .or_default()
                .push(subproblem.name.clone());
        }
    }

    for subproblems in requirement_to_subproblems.values_mut() {
        subproblems.sort();
        subproblems.dedup();
    }

    let mut correctness_arguments = problem
        .correctness_arguments
        .iter()
        .map(|argument| argument.name.clone())
        .collect::<Vec<_>>();
    correctness_arguments.sort();
    correctness_arguments.dedup();

    let mut requirement_rows = Vec::new();
    let mut covered_requirements = 0_usize;
    let mut uncovered_requirements = Vec::new();
    let mut deferred_requirements = Vec::new();

    for requirement in &problem.requirements {
        let subproblems = requirement_to_subproblems
            .get(&requirement.name)
            .cloned()
            .unwrap_or_default();

        let (state, note) = if subproblems.is_empty() {
            (
                ConcernCoverageState::Uncovered,
                "requirement is not mapped to any subproblem".to_string(),
            )
        } else if correctness_arguments.is_empty() {
            (
                ConcernCoverageState::Uncovered,
                "no correctness arguments declared".to_string(),
            )
        } else if wrspm_projection.artifacts.w_sets.is_empty()
            || wrspm_projection.artifacts.s_sets.is_empty()
            || wrspm_projection.artifacts.r_sets.is_empty()
        {
            (
                ConcernCoverageState::Deferred,
                "WRSPM W/S/R contract sets are incomplete".to_string(),
            )
        } else {
            (
                ConcernCoverageState::Covered,
                "mapped via subproblem decomposition and active correctness contract".to_string(),
            )
        };

        match state {
            ConcernCoverageState::Covered => {
                covered_requirements += 1;
            }
            ConcernCoverageState::Uncovered => {
                uncovered_requirements.push(requirement.name.clone());
            }
            ConcernCoverageState::Deferred => {
                deferred_requirements.push(requirement.name.clone());
            }
        }

        requirement_rows.push(RequirementConcernCoverage {
            requirement: requirement.name.clone(),
            subproblems,
            correctness_arguments: correctness_arguments.clone(),
            wrspm_w_sets: wrspm_projection.artifacts.w_sets.clone(),
            wrspm_s_sets: wrspm_projection.artifacts.s_sets.clone(),
            wrspm_r_sets: wrspm_projection.artifacts.r_sets.clone(),
            state,
            note,
        });
    }

    uncovered_requirements.sort();
    deferred_requirements.sort();

    let gate_status = if uncovered_requirements.is_empty() && deferred_requirements.is_empty() {
        ConcernCoverageGateStatus::Pass
    } else {
        ConcernCoverageGateStatus::Fail
    };

    ConcernCoverageSummary {
        total_requirements: problem.requirements.len(),
        covered_requirements,
        uncovered_requirements,
        deferred_requirements,
        requirement_rows,
        gate_status,
        wrspm_p_status: wrspm_projection.artifacts.p_status,
        wrspm_m_status: wrspm_projection.artifacts.m_status,
        wrspm_unresolved: wrspm_projection.unresolved,
    }
}

pub fn generate_markdown(problem: &Problem) -> String {
    let summary = summarize(problem);
    let mut output = String::new();

    output.push_str(&format!("# Concern Coverage Report: {}\n\n", problem.name));
    output.push_str(&format!(
        "- Concern coverage status: {}\n",
        gate_status_label(&summary.gate_status)
    ));
    output.push_str(&format!(
        "- Covered requirements: {}/{}\n",
        summary.covered_requirements, summary.total_requirements
    ));
    output.push_str(&format!(
        "- Uncovered requirements: {}\n",
        summary.uncovered_requirements.len()
    ));
    output.push_str(&format!(
        "- Deferred requirements: {}\n",
        summary.deferred_requirements.len()
    ));
    output.push_str(&format!("- WRSPM P status: {}\n", summary.wrspm_p_status));
    output.push_str(&format!("- WRSPM M status: {}\n", summary.wrspm_m_status));
    output.push_str(&format!(
        "- WRSPM unresolved entries: {}\n\n",
        summary.wrspm_unresolved.len()
    ));

    output.push_str("## Requirement -> Concern Coverage Matrix\n");
    output.push_str(
        "| Requirement | Subproblems | Correctness Arguments | WRSPM Contract (W/S/R) | Status | Note |\n",
    );
    output.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for row in &summary.requirement_rows {
        output.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            row.requirement,
            join_or_dash(&row.subproblems),
            join_or_dash(&row.correctness_arguments),
            format_contract_sets(row),
            state_label(&row.state),
            row.note
        ));
    }
    output.push('\n');

    output.push_str("## Explicit Uncovered Entries\n");
    append_requirement_list(
        &mut output,
        &summary.requirement_rows,
        ConcernCoverageState::Uncovered,
    );
    output.push('\n');

    output.push_str("## Explicit Deferred Entries\n");
    append_requirement_list(
        &mut output,
        &summary.requirement_rows,
        ConcernCoverageState::Deferred,
    );
    output.push('\n');

    output.push_str("## WRSPM Unresolved Contract Entries\n");
    if summary.wrspm_unresolved.is_empty() {
        output.push_str("- None.\n");
    } else {
        for entry in &summary.wrspm_unresolved {
            output.push_str(&format!("- {}\n", entry));
        }
    }
    output.push('\n');

    output
}

fn gate_status_label(status: &ConcernCoverageGateStatus) -> &'static str {
    match status {
        ConcernCoverageGateStatus::Pass => "PASS",
        ConcernCoverageGateStatus::Fail => "FAIL",
    }
}

fn state_label(state: &ConcernCoverageState) -> &'static str {
    match state {
        ConcernCoverageState::Covered => "covered",
        ConcernCoverageState::Uncovered => "uncovered",
        ConcernCoverageState::Deferred => "deferred",
    }
}

fn join_or_dash(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else {
        values.join(", ")
    }
}

fn format_contract_sets(row: &RequirementConcernCoverage) -> String {
    format!(
        "W:{}; S:{}; R:{}",
        join_or_dash(&row.wrspm_w_sets),
        join_or_dash(&row.wrspm_s_sets),
        join_or_dash(&row.wrspm_r_sets)
    )
}

fn append_requirement_list(
    output: &mut String,
    rows: &[RequirementConcernCoverage],
    state: ConcernCoverageState,
) {
    let mut matched = rows
        .iter()
        .filter(|row| row.state == state)
        .collect::<Vec<_>>();
    matched.sort_by(|left, right| left.requirement.cmp(&right.requirement));

    if matched.is_empty() {
        output.push_str("- None.\n");
        return;
    }

    for row in matched {
        output.push_str(&format!("- {}: {}\n", row.requirement, row.note));
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_markdown, summarize, ConcernCoverageGateStatus};
    use crate::parser::parse;

    #[test]
    fn concern_coverage_reports_uncovered_requirements() {
        let input = r#"
problem: Coverage

domain M kind causal role machine
domain A kind causal role given

interface "M-A" connects M, A {
  shared: {
    phenomenon Control : event [M -> A] controlledBy M
  }
}

requirement "R_covered" {
  frame: RequiredBehavior
  constrains: A
}

requirement "R_uncovered" {
  frame: RequiredBehavior
  constrains: A
}

subproblem ControlLoop {
  machine: M
  participants: M, A
  requirements: "R_covered"
}

worldProperties W_base {
  assert "world stable"
}

specification S_base {
  assert "machine controls [[M-A.Control]]"
}

requirementAssertions R_base {
  assert "requirement set exists"
}

correctnessArgument A1 {
  prove S_base and W_base entail R_base
}
"#;

        let problem = parse(input).expect("parse should succeed");
        let summary = summarize(&problem);
        assert_eq!(summary.total_requirements, 2);
        assert_eq!(summary.covered_requirements, 1);
        assert_eq!(
            summary.uncovered_requirements,
            vec!["R_uncovered".to_string()]
        );
        assert_eq!(summary.gate_status, ConcernCoverageGateStatus::Fail);

        let markdown = generate_markdown(&problem);
        assert!(markdown.contains("| R_uncovered | - | A1 |"));
        assert!(markdown.contains("## Explicit Uncovered Entries"));
        assert!(markdown.contains("- R_uncovered: requirement is not mapped to any subproblem"));
    }

    #[test]
    fn concern_coverage_includes_wrspm_contract_placeholders() {
        let input = r#"
problem: CoveragePass

domain M kind causal role machine
domain A kind causal role given

interface "M-A" connects M, A {
  shared: {
    phenomenon Control : event [M -> A] controlledBy M
  }
}

requirement "R1" {
  frame: RequiredBehavior
  constrains: A
}

subproblem ControlLoop {
  machine: M
  participants: M, A
  requirements: "R1"
}

worldProperties W_base {
  assert "world stable"
}

specification S_base {
  assert "machine controls [[M-A.Control]]"
}

requirementAssertions R_base {
  assert "goal achieved"
}

correctnessArgument A1 {
  prove S_base and W_base entail R_base
}
"#;

        let problem = parse(input).expect("parse should succeed");
        let summary = summarize(&problem);
        assert_eq!(summary.gate_status, ConcernCoverageGateStatus::Pass);

        let markdown = generate_markdown(&problem);
        assert!(markdown.contains("- WRSPM P status: deferred:"));
        assert!(markdown.contains("- WRSPM M status: deferred:"));
        assert!(markdown.contains("eh partition is deferred"));
        assert!(markdown.contains("sh partition is deferred"));
    }
}
