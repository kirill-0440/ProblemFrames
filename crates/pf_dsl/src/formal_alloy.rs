use crate::ast::*;
use std::fmt::Write;

pub fn generate_alloy(problem: &Problem) -> String {
    let mut output = String::new();
    let module_name = sanitize_name(&problem.name);

    writeln!(&mut output, "module {}", module_name).unwrap();
    writeln!(&mut output).unwrap();
    writeln!(&mut output, "abstract sig Domain {{}}").unwrap();
    writeln!(&mut output).unwrap();

    for domain in &problem.domains {
        writeln!(
            &mut output,
            "one sig {} extends Domain {{}}",
            sanitize_name(&domain.name)
        )
        .unwrap();
    }

    writeln!(&mut output).unwrap();
    writeln!(
        &mut output,
        "abstract sig Phenomenon {{ from, to, controlledBy: one Domain }}"
    )
    .unwrap();
    for (index, interface) in problem.interfaces.iter().enumerate() {
        for (phenomenon_index, phenomenon) in interface.shared_phenomena.iter().enumerate() {
            let symbol = format!(
                "Phen_{}_{}_{}",
                index,
                phenomenon_index,
                sanitize_name(&phenomenon.name)
            );
            writeln!(&mut output, "one sig {} extends Phenomenon {{}}", symbol).unwrap();
            writeln!(
                &mut output,
                "fact {}_mapping {{ {}.from = {} and {}.to = {} and {}.controlledBy = {} }}",
                symbol,
                symbol,
                sanitize_name(&phenomenon.from.name),
                symbol,
                sanitize_name(&phenomenon.to.name),
                symbol,
                sanitize_name(&phenomenon.controlled_by.name),
            )
            .unwrap();
        }
    }

    writeln!(&mut output).unwrap();
    writeln!(&mut output, "// Requirement metadata").unwrap();
    for requirement in &problem.requirements {
        writeln!(
            &mut output,
            "// - {} [{}]",
            requirement.name,
            frame_name(&requirement.frame)
        )
        .unwrap();
    }

    if !problem.correctness_arguments.is_empty() {
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "// Proof obligations").unwrap();
    }
    for argument in &problem.correctness_arguments {
        let pred_name = format!("Obl_{}", sanitize_name(&argument.name));
        writeln!(&mut output, "pred {} {{", pred_name).unwrap();
        writeln!(
            &mut output,
            "  // {} and {} entail {}",
            argument.specification_set, argument.world_set, argument.requirement_set
        )
        .unwrap();

        if let Some(spec_set) = problem.assertion_sets.iter().find(|set| {
            set.name == argument.specification_set
                && matches!(set.scope, AssertionScope::Specification)
        }) {
            for assertion in &spec_set.assertions {
                writeln!(&mut output, "  // S: {}", assertion.text).unwrap();
            }
        }
        if let Some(world_set) = problem.assertion_sets.iter().find(|set| {
            set.name == argument.world_set && matches!(set.scope, AssertionScope::WorldProperties)
        }) {
            for assertion in &world_set.assertions {
                writeln!(&mut output, "  // W: {}", assertion.text).unwrap();
            }
        }
        if let Some(req_set) = problem.assertion_sets.iter().find(|set| {
            set.name == argument.requirement_set
                && matches!(set.scope, AssertionScope::RequirementAssertions)
        }) {
            for assertion in &req_set.assertions {
                writeln!(&mut output, "  // R: {}", assertion.text).unwrap();
            }
        }
        writeln!(&mut output, "}}").unwrap();
        writeln!(&mut output, "run {} for 6", pred_name).unwrap();
    }

    output
}

fn frame_name(frame: &FrameType) -> &str {
    match frame {
        FrameType::RequiredBehavior => "RequiredBehavior",
        FrameType::CommandedBehavior => "CommandedBehavior",
        FrameType::InformationDisplay => "InformationDisplay",
        FrameType::SimpleWorkpieces => "SimpleWorkpieces",
        FrameType::Transformation => "Transformation",
        FrameType::Custom(name) => name.as_str(),
    }
}

fn sanitize_name(name: &str) -> String {
    let mut output = String::new();
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.is_empty() {
        "unnamed".to_string()
    } else {
        output
    }
}

#[cfg(test)]
mod tests {
    use super::generate_alloy;
    use crate::ast::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn domain(name: &str, kind: DomainKind, role: DomainRole) -> Domain {
        Domain {
            name: name.to_string(),
            kind,
            role,
            span: span(),
            source_path: None,
        }
    }

    fn reference(name: &str) -> Reference {
        Reference {
            name: name.to_string(),
            span: span(),
        }
    }

    #[test]
    fn emits_alloy_with_obligations() {
        let problem = Problem {
            name: "Thermostat".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Room", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![Interface {
                name: "M-R".to_string(),
                connects: vec![reference("Machine"), reference("Room")],
                shared_phenomena: vec![Phenomenon {
                    name: "Observe".to_string(),
                    type_: PhenomenonType::Value,
                    from: reference("Room"),
                    to: reference("Machine"),
                    controlled_by: reference("Room"),
                    span: span(),
                }],
                span: span(),
                source_path: None,
            }],
            requirements: vec![Requirement {
                name: "R1".to_string(),
                frame: FrameType::RequiredBehavior,
                phenomena: vec![],
                constraint: "maintain room".to_string(),
                constrains: Some(reference("Room")),
                reference: None,
                span: span(),
                source_path: None,
            }],
            assertion_sets: vec![
                AssertionSet {
                    name: "S_control".to_string(),
                    scope: AssertionScope::Specification,
                    assertions: vec![Assertion {
                        text: "controller observes room".to_string(),
                        language: None,
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "W_base".to_string(),
                    scope: AssertionScope::WorldProperties,
                    assertions: vec![Assertion {
                        text: "room physics is stable".to_string(),
                        language: None,
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "R_goal".to_string(),
                    scope: AssertionScope::RequirementAssertions,
                    assertions: vec![Assertion {
                        text: "room reaches target".to_string(),
                        language: None,
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A1".to_string(),
                specification_set: "S_control".to_string(),
                world_set: "W_base".to_string(),
                requirement_set: "R_goal".to_string(),
                span: span(),
                source_path: None,
            }],
        };

        let alloy = generate_alloy(&problem);
        assert!(alloy.contains("module Thermostat"));
        assert!(alloy.contains("one sig Machine extends Domain"));
        assert!(alloy.contains("pred Obl_A1"));
        assert!(alloy.contains("S_control and W_base entail R_goal"));
    }
}
