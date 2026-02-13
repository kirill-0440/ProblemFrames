use crate::ast::Problem;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecompositionClosureSummary {
    pub total_requirements: usize,
    pub mapped_requirements: usize,
    pub uncovered_requirements: Vec<String>,
    pub total_subproblems: usize,
    pub subproblems_without_requirements: Vec<String>,
    pub subproblems_without_machine: Vec<String>,
}

impl DecompositionClosureSummary {
    pub fn closure_passes(&self) -> bool {
        self.uncovered_requirements.is_empty()
            && self.subproblems_without_requirements.is_empty()
            && self.subproblems_without_machine.is_empty()
    }
}

pub fn summarize(problem: &Problem) -> DecompositionClosureSummary {
    let mut covered_requirements = HashSet::new();
    let mut subproblems_without_requirements = Vec::new();
    let mut subproblems_without_machine = Vec::new();

    for subproblem in &problem.subproblems {
        if subproblem.requirements.is_empty() {
            subproblems_without_requirements.push(subproblem.name.clone());
        }
        if subproblem.machine.is_none() {
            subproblems_without_machine.push(subproblem.name.clone());
        }
        for requirement in &subproblem.requirements {
            covered_requirements.insert(requirement.name.as_str());
        }
    }

    let mut uncovered_requirements = problem
        .requirements
        .iter()
        .filter(|requirement| !covered_requirements.contains(requirement.name.as_str()))
        .map(|requirement| requirement.name.clone())
        .collect::<Vec<_>>();

    uncovered_requirements.sort();
    subproblems_without_requirements.sort();
    subproblems_without_machine.sort();

    DecompositionClosureSummary {
        total_requirements: problem.requirements.len(),
        mapped_requirements: problem.requirements.len() - uncovered_requirements.len(),
        uncovered_requirements,
        total_subproblems: problem.subproblems.len(),
        subproblems_without_requirements,
        subproblems_without_machine,
    }
}

pub fn generate_markdown(problem: &Problem) -> String {
    let summary = summarize(problem);
    let mut requirement_to_subproblems: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for subproblem in &problem.subproblems {
        for requirement in &subproblem.requirements {
            requirement_to_subproblems
                .entry(requirement.name.clone())
                .or_default()
                .push(subproblem.name.clone());
        }
    }

    let mut output = String::new();

    output.push_str(&format!(
        "# Decomposition Closure Report: {}\n\n",
        problem.name
    ));
    output.push_str(&format!(
        "- Total requirements: {}\n",
        summary.total_requirements
    ));
    output.push_str(&format!(
        "- Mapped requirements: {}\n",
        summary.mapped_requirements
    ));
    output.push_str(&format!(
        "- Uncovered requirements: {}\n",
        summary.uncovered_requirements.len()
    ));
    output.push_str(&format!(
        "- Total subproblems: {}\n",
        summary.total_subproblems
    ));
    output.push_str(&format!(
        "- Subproblems without requirements: {}\n",
        summary.subproblems_without_requirements.len()
    ));
    output.push_str(&format!(
        "- Subproblems without machine: {}\n",
        summary.subproblems_without_machine.len()
    ));
    output.push_str(&format!(
        "- Closure status: {}\n\n",
        if summary.closure_passes() {
            "PASS"
        } else {
            "FAIL"
        }
    ));

    output.push_str("## Requirement Coverage Matrix\n");
    output.push_str("| Requirement | Subproblems | Status |\n");
    output.push_str("| --- | --- | --- |\n");
    for requirement in &problem.requirements {
        let linked_subproblems = requirement_to_subproblems
            .get(&requirement.name)
            .cloned()
            .unwrap_or_default();
        let mut linked_subproblems = linked_subproblems;
        linked_subproblems.sort();
        let status = if linked_subproblems.is_empty() {
            "uncovered"
        } else {
            "covered"
        };
        let subproblems = if linked_subproblems.is_empty() {
            "-".to_string()
        } else {
            linked_subproblems.join(", ")
        };
        output.push_str(&format!(
            "| {} | {} | {} |\n",
            requirement.name, subproblems, status
        ));
    }
    output.push('\n');

    output.push_str("## Uncovered Requirements\n");
    if summary.uncovered_requirements.is_empty() {
        output.push_str("- (none)\n");
    } else {
        for requirement in &summary.uncovered_requirements {
            output.push_str(&format!("- {}\n", requirement));
        }
    }
    output.push('\n');

    output.push_str("### Orphan Subproblems\n");
    if summary.subproblems_without_requirements.is_empty() {
        output.push_str("- None.\n");
    } else {
        for subproblem in &summary.subproblems_without_requirements {
            output.push_str(&format!("- {}\n", subproblem));
        }
    }
    output.push('\n');

    output.push_str("### Subproblems Without Machine\n");
    if summary.subproblems_without_machine.is_empty() {
        output.push_str("- None.\n");
    } else {
        for subproblem in &summary.subproblems_without_machine {
            output.push_str(&format!("- {}\n", subproblem));
        }
    }
    output.push('\n');

    output
}

#[cfg(test)]
mod tests {
    use super::{generate_markdown, summarize};
    use crate::parser::parse;

    #[test]
    fn decomposition_summary_reports_uncovered_requirement() {
        let input = r#"
problem: CoverageExample

domain Tool kind causal role machine
domain User kind biddable role given
domain Data kind lexical role given

interface "T-U" connects Tool, User {
  shared: { phenomenon Ask : command [User -> Tool] controlledBy User }
}

interface "T-D" connects Tool, Data {
  shared: { phenomenon Write : value [Tool -> Data] controlledBy Tool }
}

requirement "R1" {
  frame: SimpleWorkpieces
  constraint: "R1 is covered"
  constrains: Data
  reference: User
}

requirement "R2" {
  frame: SimpleWorkpieces
  constraint: "R2 is uncovered"
  constrains: Data
  reference: User
}

subproblem S1 {
  machine: Tool
  participants: Tool, User, Data
  requirements: "R1"
}
"#;
        let problem = parse(input).expect("parse must succeed");
        let summary = summarize(&problem);
        assert_eq!(summary.total_requirements, 2);
        assert_eq!(summary.mapped_requirements, 1);
        assert_eq!(summary.uncovered_requirements, vec!["R2".to_string()]);
        assert!(!summary.closure_passes());
    }

    #[test]
    fn decomposition_markdown_emits_pass_when_complete() {
        let input = r#"
problem: Complete

domain Tool kind causal role machine
domain User kind biddable role given
domain Data kind lexical role given

interface "T-U" connects Tool, User {
  shared: { phenomenon Ask : command [User -> Tool] controlledBy User }
}

interface "T-D" connects Tool, Data {
  shared: { phenomenon Write : value [Tool -> Data] controlledBy Tool }
}

requirement "R1" {
  frame: SimpleWorkpieces
  constraint: "covered"
  constrains: Data
  reference: User
}

subproblem S1 {
  machine: Tool
  participants: Tool, User, Data
  requirements: "R1"
}
"#;
        let problem = parse(input).expect("parse must succeed");
        let markdown = generate_markdown(&problem);
        assert!(markdown.contains("Closure status: PASS"));
        assert!(markdown.contains("Uncovered requirements: 0"));
    }
}
