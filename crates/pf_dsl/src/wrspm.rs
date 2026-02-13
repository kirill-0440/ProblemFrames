use crate::ast::{AssertionScope, DomainRole, Problem};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WrspmProjection {
    pub problem: String,
    pub machine_domain: Option<String>,
    pub artifacts: WrspmArtifacts,
    pub interface_phenomena: Vec<WrspmPhenomenon>,
    pub obligations: Vec<WrspmObligation>,
    pub unresolved: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WrspmArtifacts {
    pub w_sets: Vec<String>,
    pub r_sets: Vec<String>,
    pub s_sets: Vec<String>,
    pub p_status: String,
    pub m_status: String,
}

#[derive(Debug, Serialize)]
pub struct WrspmPhenomenon {
    pub interface: String,
    pub name: String,
    pub controlled_by: String,
    pub partition: String,
}

#[derive(Debug, Serialize)]
pub struct WrspmObligation {
    pub argument: String,
    pub expression: String,
}

pub fn project(problem: &Problem) -> WrspmProjection {
    let machine_domain = problem
        .domains
        .iter()
        .find(|domain| domain.role == DomainRole::Machine)
        .map(|domain| domain.name.clone());

    let w_sets = problem
        .assertion_sets
        .iter()
        .filter(|set| set.scope == AssertionScope::WorldProperties)
        .map(|set| set.name.clone())
        .collect::<Vec<_>>();
    let r_sets = problem
        .assertion_sets
        .iter()
        .filter(|set| set.scope == AssertionScope::RequirementAssertions)
        .map(|set| set.name.clone())
        .collect::<Vec<_>>();
    let s_sets = problem
        .assertion_sets
        .iter()
        .filter(|set| set.scope == AssertionScope::Specification)
        .map(|set| set.name.clone())
        .collect::<Vec<_>>();

    let mut interface_phenomena = Vec::new();
    for interface in &problem.interfaces {
        for phenomenon in &interface.shared_phenomena {
            let partition = if machine_domain
                .as_ref()
                .map(|machine| phenomenon.controlled_by.name == *machine)
                .unwrap_or(false)
            {
                "sv"
            } else {
                "ev"
            };
            interface_phenomena.push(WrspmPhenomenon {
                interface: interface.name.clone(),
                name: phenomenon.name.clone(),
                controlled_by: phenomenon.controlled_by.name.clone(),
                partition: partition.to_string(),
            });
        }
    }

    let obligations = problem
        .correctness_arguments
        .iter()
        .map(|argument| WrspmObligation {
            argument: argument.name.clone(),
            expression: format!(
                "{} and {} entail {}",
                argument.specification_set, argument.world_set, argument.requirement_set
            ),
        })
        .collect::<Vec<_>>();

    WrspmProjection {
        problem: problem.name.clone(),
        machine_domain: machine_domain.clone(),
        artifacts: WrspmArtifacts {
            w_sets,
            r_sets,
            s_sets,
            p_status: "deferred: program realization metadata not modeled in PF AST".to_string(),
            m_status: "deferred: platform realization metadata not modeled in PF AST".to_string(),
        },
        interface_phenomena,
        obligations,
        unresolved: vec![
            "eh partition is deferred in phase 1 (hidden environment phenomena)".to_string(),
            "sh partition is deferred in phase 1 (hidden system phenomena)".to_string(),
        ],
    }
}

pub fn generate_markdown(problem: &Problem) -> String {
    let projection = project(problem);
    let mut output = String::new();

    output.push_str(&format!("# WRSPM Report: {}\n\n", projection.problem));
    output.push_str("## 1. Artifact Projection (W/R/S/P/M)\n");
    output.push_str(&format!(
        "- Machine domain: {}\n",
        machine_name(&projection)
    ));
    output.push_str(&format!(
        "- W sets: {}\n",
        join_or_none(&projection.artifacts.w_sets)
    ));
    output.push_str(&format!(
        "- R sets: {}\n",
        join_or_none(&projection.artifacts.r_sets)
    ));
    output.push_str(&format!(
        "- S sets: {}\n",
        join_or_none(&projection.artifacts.s_sets)
    ));
    output.push_str(&format!("- P: {}\n", projection.artifacts.p_status));
    output.push_str(&format!("- M: {}\n\n", projection.artifacts.m_status));

    output.push_str("## 2. Interface Phenomena Partition (ev/sv)\n");
    if projection.interface_phenomena.is_empty() {
        output.push_str("- (none)\n\n");
    } else {
        for phenomenon in &projection.interface_phenomena {
            output.push_str(&format!(
                "- `{}`.`{}` controlledBy `{}` -> `{}`\n",
                phenomenon.interface,
                phenomenon.name,
                phenomenon.controlled_by,
                phenomenon.partition
            ));
        }
        output.push('\n');
    }

    output.push_str("## 3. Correctness Obligations\n");
    if projection.obligations.is_empty() {
        output.push_str("- (none)\n\n");
    } else {
        for obligation in &projection.obligations {
            output.push_str(&format!(
                "- `{}`: {}\n",
                obligation.argument, obligation.expression
            ));
        }
        output.push('\n');
    }

    output.push_str("## 4. Unresolved / Deferred\n");
    for item in &projection.unresolved {
        output.push_str(&format!("- {}\n", item));
    }
    output.push('\n');

    output
}

pub fn generate_json(problem: &Problem) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&project(problem))
}

fn machine_name(projection: &WrspmProjection) -> &str {
    projection
        .machine_domain
        .as_deref()
        .unwrap_or("(not declared)")
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        "(none)".to_string()
    } else {
        values.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_json, generate_markdown, project};
    use crate::parser::parse;

    #[test]
    fn wrspm_projection_partitions_ev_and_sv() {
        let input = r#"
problem: WRSPMProjection

domain User kind biddable role given
domain Tool kind causal role machine
domain Data kind lexical role given

interface "U-T" connects User, Tool {
  shared: {
    phenomenon Ask : command [User -> Tool] controlledBy User
    phenomenon Ack : event [Tool -> User] controlledBy Tool
  }
}

interface "T-D" connects Tool, Data {
  shared: {
    phenomenon Persist : value [Tool -> Data] controlledBy Tool
  }
}

requirement "R1" {
  frame: Transformation
  constraint: "store data"
  constrains: Data
}

subproblem SP1 {
  machine: Tool
  participants: Tool, Data
  requirements: "R1"
}

worldProperties W1 {
  assert "world property"
}

specification S1 {
  assert "spec property"
}

requirementAssertions R_set {
  assert "requirement property"
}

correctnessArgument A1 {
  prove S1 and W1 entail R_set
}
"#;
        let problem = parse(input).expect("parse must succeed");
        let projection = project(&problem);
        assert_eq!(projection.machine_domain.as_deref(), Some("Tool"));
        assert!(projection
            .interface_phenomena
            .iter()
            .any(|p| p.name == "Ask" && p.partition == "ev"));
        assert!(projection
            .interface_phenomena
            .iter()
            .any(|p| p.name == "Ack" && p.partition == "sv"));
        assert_eq!(projection.obligations.len(), 1);
    }

    #[test]
    fn wrspm_outputs_are_generated() {
        let input = r#"
problem: Minimal
domain Tool kind causal role machine
domain Data kind lexical role given
interface "T-D" connects Tool, Data {
  shared: { phenomenon Persist : value [Tool -> Data] controlledBy Tool }
}
requirement "R1" {
  frame: Transformation
  constraint: "store"
  constrains: Data
}
"#;
        let problem = parse(input).expect("parse must succeed");
        let markdown = generate_markdown(&problem);
        assert!(markdown.contains("WRSPM Report"));
        assert!(markdown.contains("Artifact Projection"));

        let json = generate_json(&problem).expect("json generation must succeed");
        assert!(json.contains("\"problem\": \"Minimal\""));
        assert!(json.contains("\"interface_phenomena\""));
    }
}
