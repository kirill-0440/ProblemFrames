use crate::ast::{Mark, PhenomenonType, Problem};
use crate::pim::trace_target_id;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TraceLink {
    pub source_kind: String,
    pub source_id: String,
    pub target_id: String,
    pub target_kind: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TraceMapTarget {
    pub id: String,
    pub kind: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TraceMapCoverage {
    pub status: String,
    pub generated_targets_total: usize,
    pub mapped_targets_total: usize,
    pub unmapped_generated_targets: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TraceMapReport {
    pub model: String,
    pub links: Vec<TraceLink>,
    pub generated_targets: Vec<TraceMapTarget>,
    pub coverage: TraceMapCoverage,
}

fn has_mark(marks: &[Mark], mark: &str) -> bool {
    marks.iter().any(|candidate| candidate.name == mark)
}

fn mark_value<'a>(marks: &'a [Mark], mark: &str) -> Option<&'a str> {
    marks.iter().find_map(|candidate| {
        if candidate.name == mark {
            candidate.value.as_deref()
        } else {
            None
        }
    })
}

pub fn build_trace_map(problem: &Problem) -> TraceMapReport {
    let mut links = Vec::new();
    let mut targets: BTreeMap<String, String> = BTreeMap::new();

    let mut add_target = |id: String, kind: &str| {
        targets.entry(id).or_insert_with(|| kind.to_string());
    };
    let mut add_link =
        |source_kind: &str, source_id: String, target_id: String, target_kind: &str| {
            links.push(TraceLink {
                source_kind: source_kind.to_string(),
                source_id,
                target_id,
                target_kind: target_kind.to_string(),
            });
        };

    for domain in &problem.domains {
        let block_target = trace_target_id("sysml.block", &[&domain.name]);
        add_target(block_target.clone(), "sysml.block");
        add_link("domain", domain.name.clone(), block_target, "sysml.block");

        if let Some(context) = mark_value(&domain.marks, "ddd.bounded_context") {
            let context_target = trace_target_id("ddd.context", &[context]);
            add_target(context_target.clone(), "ddd.context");
            add_link("domain", domain.name.clone(), context_target, "ddd.context");
        }
        if has_mark(&domain.marks, "ddd.aggregate_root") {
            let aggregate_target = trace_target_id("ddd.aggregate", &[&domain.name]);
            add_target(aggregate_target.clone(), "ddd.aggregate");
            add_link(
                "domain",
                domain.name.clone(),
                aggregate_target,
                "ddd.aggregate",
            );
        }
        if has_mark(&domain.marks, "ddd.value_object") {
            let value_object_target = trace_target_id("ddd.value_object", &[&domain.name]);
            add_target(value_object_target.clone(), "ddd.value_object");
            add_link(
                "domain",
                domain.name.clone(),
                value_object_target,
                "ddd.value_object",
            );
        }
        if has_mark(&domain.marks, "ddd.external_system") {
            let external_target = trace_target_id("ddd.external_system", &[&domain.name]);
            add_target(external_target.clone(), "ddd.external_system");
            add_link(
                "domain",
                domain.name.clone(),
                external_target,
                "ddd.external_system",
            );
        }
    }

    for requirement in &problem.requirements {
        let requirement_target = trace_target_id("sysml.requirement", &[&requirement.name]);
        add_target(requirement_target.clone(), "sysml.requirement");
        add_link(
            "requirement",
            requirement.name.clone(),
            requirement_target,
            "sysml.requirement",
        );

        if let Some(service) = mark_value(&requirement.marks, "ddd.application_service") {
            let service_target = trace_target_id("ddd.application_service", &[service]);
            add_target(service_target.clone(), "ddd.application_service");
            add_link(
                "requirement",
                requirement.name.clone(),
                service_target,
                "ddd.application_service",
            );
        }
    }

    for interface in &problem.interfaces {
        let interface_target = trace_target_id("sysml.interface", &[&interface.name]);
        add_target(interface_target.clone(), "sysml.interface");
        add_link(
            "interface",
            interface.name.clone(),
            interface_target,
            "sysml.interface",
        );

        for phenomenon in &interface.shared_phenomena {
            let source_id = format!("{}.{}", interface.name, phenomenon.name);
            let phenomenon_target =
                trace_target_id("sysml.phenomenon", &[&interface.name, &phenomenon.name]);
            add_target(phenomenon_target.clone(), "sysml.phenomenon");
            add_link(
                "phenomenon",
                source_id.clone(),
                phenomenon_target,
                "sysml.phenomenon",
            );

            match phenomenon.type_ {
                PhenomenonType::Command => {
                    let command_target =
                        trace_target_id("ddd.command", &[&interface.name, &phenomenon.name]);
                    add_target(command_target.clone(), "ddd.command");
                    add_link("phenomenon", source_id, command_target, "ddd.command");
                }
                PhenomenonType::Event => {
                    let event_target =
                        trace_target_id("ddd.event", &[&interface.name, &phenomenon.name]);
                    add_target(event_target.clone(), "ddd.event");
                    add_link("phenomenon", source_id, event_target, "ddd.event");
                }
                _ => {}
            }
        }
    }

    links.sort_by(|left, right| {
        left.source_kind
            .cmp(&right.source_kind)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.target_id.cmp(&right.target_id))
    });

    let generated_target_ids = targets.keys().cloned().collect::<BTreeSet<_>>();
    let mapped_target_ids = links
        .iter()
        .map(|link| link.target_id.clone())
        .collect::<BTreeSet<_>>();
    let unmapped_generated_targets = generated_target_ids
        .difference(&mapped_target_ids)
        .cloned()
        .collect::<Vec<_>>();

    let generated_targets = targets
        .into_iter()
        .map(|(id, kind)| TraceMapTarget { id, kind })
        .collect::<Vec<_>>();

    let coverage = TraceMapCoverage {
        status: if unmapped_generated_targets.is_empty() {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
        generated_targets_total: generated_target_ids.len(),
        mapped_targets_total: mapped_target_ids.len(),
        unmapped_generated_targets,
    };

    TraceMapReport {
        model: problem.name.clone(),
        links,
        generated_targets,
        coverage,
    }
}

pub fn generate_trace_map_json(problem: &Problem) -> Result<String, serde_json::Error> {
    let report = build_trace_map(problem);
    serde_json::to_string_pretty(&report)
}

#[cfg(test)]
mod tests {
    use super::{build_trace_map, generate_trace_map_json};
    use crate::parser::parse;

    #[test]
    fn trace_map_reports_pass_for_mapped_targets() {
        let input = r#"
            problem: TraceMap
            domain Tool kind causal role machine
            domain Ledger kind lexical role given marks: {
                @ddd.bounded_context("Accounting")
            }
            interface "Tool-Ledger" connects Tool, Ledger {
                shared: {
                    phenomenon Execute : command [Tool -> Ledger] controlledBy Tool
                }
            }
            requirement "R1" {
                frame: Transformation
                constrains: Ledger
                marks: {
                    @ddd.application_service("PersistLedgerEntry")
                }
            }
        "#;
        let problem = parse(input).expect("parse failed");
        let report = build_trace_map(&problem);
        assert_eq!(report.coverage.status, "PASS");
        assert!(report
            .generated_targets
            .iter()
            .any(|target| target.id == "ddd.command.tool_ledger.execute"));
    }

    #[test]
    fn trace_map_json_is_generated() {
        let input = r#"
            problem: TraceMapJson
            domain Tool kind causal role machine
            domain Operator kind biddable role given
            interface "Tool-Operator" connects Tool, Operator {
                shared: {
                    phenomenon Notify : event [Tool -> Operator] controlledBy Tool
                }
            }
            requirement "R1" {
                frame: InformationDisplay
                reference: Operator
                constrains: Operator
            }
        "#;
        let problem = parse(input).expect("parse failed");
        let json = generate_trace_map_json(&problem).expect("json generation failed");
        assert!(json.contains("\"coverage\""));
        assert!(json.contains("\"status\": \"PASS\""));
    }
}
