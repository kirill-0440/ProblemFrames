use crate::ast::{FrameType, Mark, PhenomenonType, Problem};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

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

fn sanitize_identifier(value: &str) -> String {
    let mut normalized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();

    while normalized.contains("__") {
        normalized = normalized.replace("__", "_");
    }

    normalized.trim_matches('_').to_string()
}

pub fn generate_ddd_pim_markdown(problem: &Problem) -> String {
    let mut output = String::new();
    output.push_str(&format!("# DDD PIM Report: {}\n\n", problem.name));

    let mut context_to_domains: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut aggregate_candidates = Vec::new();
    let mut value_object_candidates = Vec::new();
    let mut external_systems = Vec::new();

    for domain in &problem.domains {
        if let Some(context) = mark_value(&domain.marks, "ddd.bounded_context") {
            context_to_domains
                .entry(context.to_string())
                .or_default()
                .push(domain.name.clone());
        }
        if has_mark(&domain.marks, "ddd.aggregate_root") {
            aggregate_candidates.push(domain.name.clone());
        }
        if has_mark(&domain.marks, "ddd.value_object") {
            value_object_candidates.push(domain.name.clone());
        }
        if has_mark(&domain.marks, "ddd.external_system") {
            external_systems.push(domain.name.clone());
        }
    }

    output.push_str("## Bounded Context Map\n");
    if context_to_domains.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for domains in context_to_domains.values_mut() {
            domains.sort();
            domains.dedup();
        }
        for (context, domains) in context_to_domains {
            output.push_str(&format!("- {}: {}\n", context, domains.join(", ")));
        }
        output.push('\n');
    }

    let mut commands = BTreeSet::new();
    let mut events = BTreeSet::new();
    for interface in &problem.interfaces {
        for phenomenon in &interface.shared_phenomena {
            let token = format!(
                "{}.{} ({} -> {})",
                interface.name, phenomenon.name, phenomenon.from.name, phenomenon.to.name
            );
            match phenomenon.type_ {
                PhenomenonType::Command => {
                    commands.insert(token);
                }
                PhenomenonType::Event => {
                    events.insert(token);
                }
                _ => {}
            }
        }
    }

    output.push_str("## Command Inventory\n");
    if commands.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for command in commands {
            output.push_str(&format!("- {}\n", command));
        }
        output.push('\n');
    }

    output.push_str("## Event Inventory\n");
    if events.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for event in events {
            output.push_str(&format!("- {}\n", event));
        }
        output.push('\n');
    }

    aggregate_candidates.sort();
    aggregate_candidates.dedup();
    value_object_candidates.sort();
    value_object_candidates.dedup();
    external_systems.sort();
    external_systems.dedup();

    output.push_str("## Aggregate Candidates\n");
    if aggregate_candidates.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for aggregate in aggregate_candidates {
            output.push_str(&format!("- {}\n", aggregate));
        }
        output.push('\n');
    }

    output.push_str("## Value Object Candidates\n");
    if value_object_candidates.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for value_object in value_object_candidates {
            output.push_str(&format!("- {}\n", value_object));
        }
        output.push('\n');
    }

    output.push_str("## External Systems\n");
    if external_systems.is_empty() {
        output.push_str("- None.\n\n");
    } else {
        for external in external_systems {
            output.push_str(&format!("- {}\n", external));
        }
        output.push('\n');
    }

    let mut services = BTreeSet::new();
    for requirement in &problem.requirements {
        if let Some(service) = mark_value(&requirement.marks, "ddd.application_service") {
            services.insert(format!("{} ({})", service, requirement.name));
        }
    }

    output.push_str("## Application Service Candidates\n");
    if services.is_empty() {
        output.push_str("- None.\n");
    } else {
        for service in services {
            output.push_str(&format!("- {}\n", service));
        }
    }

    output
}

pub fn generate_sysml2_text(problem: &Problem) -> String {
    let mut output = String::new();
    output.push_str(&format!("package {} {{\n", problem.name));

    let mut requirements = problem.requirements.clone();
    requirements.sort_by(|left, right| left.name.cmp(&right.name));
    for requirement in requirements {
        output.push_str(&format!(
            "  requirement {} \"{}\";\n",
            requirement.name, requirement.constraint
        ));
    }

    let mut domains = problem.domains.clone();
    domains.sort_by(|left, right| left.name.cmp(&right.name));
    for domain in domains {
        output.push_str(&format!(
            "  block {} /* kind={:?}, role={:?} */;\n",
            domain.name, domain.kind, domain.role
        ));
    }

    let mut interfaces = problem.interfaces.clone();
    interfaces.sort_by(|left, right| left.name.cmp(&right.name));
    for interface in interfaces {
        output.push_str(&format!("  interface {} {{\n", interface.name));
        for phenomenon in interface.shared_phenomena {
            output.push_str(&format!(
                "    phenomenon {} : {:?} from {} to {} controlledBy {};\n",
                phenomenon.name,
                phenomenon.type_,
                phenomenon.from.name,
                phenomenon.to.name,
                phenomenon.controlled_by.name
            ));
        }
        output.push_str("  }\n");
    }

    output.push_str("}\n");
    output
}

pub fn generate_sysml2_json(problem: &Problem) -> Result<String, serde_json::Error> {
    let mut requirements = problem
        .requirements
        .iter()
        .map(|requirement| {
            json!({
                "id": requirement.name,
                "constraint": requirement.constraint,
                "frame": format!("{:?}", requirement.frame),
                "marks": requirement
                    .marks
                    .iter()
                    .map(|mark| json!({"name": mark.name, "value": mark.value}))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    requirements.sort_by_key(|entry| entry["id"].as_str().unwrap_or_default().to_string());

    let mut blocks = problem
        .domains
        .iter()
        .map(|domain| {
            json!({
                "id": domain.name,
                "kind": format!("{:?}", domain.kind),
                "role": format!("{:?}", domain.role),
                "marks": domain
                    .marks
                    .iter()
                    .map(|mark| json!({"name": mark.name, "value": mark.value}))
                    .collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    blocks.sort_by_key(|entry| entry["id"].as_str().unwrap_or_default().to_string());

    let mut interfaces = problem
        .interfaces
        .iter()
        .map(|interface| {
            let mut phenomena = interface
                .shared_phenomena
                .iter()
                .map(|phenomenon| {
                    json!({
                        "id": phenomenon.name,
                        "type": format!("{:?}", phenomenon.type_),
                        "from": phenomenon.from.name,
                        "to": phenomenon.to.name,
                        "controlled_by": phenomenon.controlled_by.name,
                    })
                })
                .collect::<Vec<_>>();
            phenomena.sort_by_key(|entry| entry["id"].as_str().unwrap_or_default().to_string());

            json!({
                "id": interface.name,
                "connects": interface.connects.iter().map(|reference| reference.name.clone()).collect::<Vec<_>>(),
                "phenomena": phenomena,
            })
        })
        .collect::<Vec<_>>();
    interfaces.sort_by_key(|entry| entry["id"].as_str().unwrap_or_default().to_string());

    let payload = json!({
        "model": problem.name,
        "target": "sysml-v2-json",
        "schema_version": "0.1-draft",
        "requirements": requirements,
        "blocks": blocks,
        "interfaces": interfaces,
        "frames": problem
            .requirements
            .iter()
            .map(|requirement| {
                json!({
                    "requirement_id": requirement.name,
                    "frame_type": match &requirement.frame {
                        FrameType::Custom(name) => name.clone(),
                        _ => format!("{:?}", requirement.frame),
                    },
                })
            })
            .collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&payload)
}

pub fn trace_target_id(prefix: &str, parts: &[&str]) -> String {
    let normalized = parts
        .iter()
        .map(|part| sanitize_identifier(part))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        prefix.to_string()
    } else {
        format!("{}.{}", prefix, normalized.join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_ddd_pim_markdown, generate_sysml2_json, generate_sysml2_text};
    use crate::parser::parse;

    #[test]
    fn generates_ddd_report_with_context_and_services() {
        let input = r#"
            problem: Pim
            domain Tool kind causal role machine
            domain Payments kind causal role given marks: {
                @ddd.bounded_context("Payments")
                @ddd.aggregate_root
            }
            domain User kind biddable role given
            interface "Tool-Payments" connects Tool, Payments {
                shared: {
                    phenomenon Execute : command [Tool -> Payments] controlledBy Tool
                    phenomenon Updated : event [Payments -> Tool] controlledBy Payments
                }
            }
            interface "Tool-User" connects Tool, User {
                shared: {
                    phenomenon Show : event [Tool -> User] controlledBy Tool
                }
            }
            requirement "R1" {
                frame: InformationDisplay
                reference: User
                constrains: Payments
                marks: {
                    @ddd.application_service("DisplayState")
                }
            }
        "#;
        let problem = parse(input).expect("parse failed");
        let report = generate_ddd_pim_markdown(&problem);
        assert!(report.contains("Payments: Payments"));
        assert!(report.contains("Execute"));
        assert!(report.contains("DisplayState (R1)"));
    }

    #[test]
    fn generates_sysml_text_and_json_outputs() {
        let input = r#"
            problem: PimSysml
            domain Tool kind causal role machine
            domain Ledger kind lexical role given
            interface "Tool-Ledger" connects Tool, Ledger {
                shared: {
                    phenomenon Persist : value [Tool -> Ledger] controlledBy Tool
                }
            }
            requirement "R1" {
                frame: Transformation
                constrains: Ledger
            }
        "#;
        let problem = parse(input).expect("parse failed");
        let text = generate_sysml2_text(&problem);
        let json = generate_sysml2_json(&problem).expect("json generation failed");
        assert!(text.contains("requirement R1"));
        assert!(text.contains("block Ledger"));
        assert!(json.contains("\"target\": \"sysml-v2-json\""));
        assert!(json.contains("\"id\": \"R1\""));
    }
}
