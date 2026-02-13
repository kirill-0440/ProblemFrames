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

    if !problem.subproblems.is_empty() {
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "// Subproblem decomposition").unwrap();
        for subproblem in &problem.subproblems {
            let machine = subproblem
                .machine
                .as_ref()
                .map(|machine| machine.name.as_str())
                .unwrap_or("<missing>");
            let participants = subproblem
                .participants
                .iter()
                .map(|participant| participant.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let requirements = subproblem
                .requirements
                .iter()
                .map(|requirement| requirement.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                &mut output,
                "// - {}: machine={}, participants=[{}], requirements=[{}]",
                subproblem.name, machine, participants, requirements
            )
            .unwrap();
        }
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

        let spec_alloy = find_alloy_assertions(
            problem,
            &argument.specification_set,
            AssertionScope::Specification,
        );
        let world_alloy = find_alloy_assertions(
            problem,
            &argument.world_set,
            AssertionScope::WorldProperties,
        );
        let req_alloy = find_alloy_assertions(
            problem,
            &argument.requirement_set,
            AssertionScope::RequirementAssertions,
        );

        if !spec_alloy.is_empty() || !world_alloy.is_empty() || !req_alloy.is_empty() {
            for assertion in &spec_alloy {
                writeln!(&mut output, "  ({})", assertion.text).unwrap();
            }
            for assertion in &world_alloy {
                writeln!(&mut output, "  ({})", assertion.text).unwrap();
            }

            if !req_alloy.is_empty() {
                if req_alloy.len() == 1 {
                    writeln!(&mut output, "  not ({})", req_alloy[0].text).unwrap();
                } else {
                    writeln!(&mut output, "  not (").unwrap();
                    for (index, assertion) in req_alloy.iter().enumerate() {
                        let suffix = if index + 1 == req_alloy.len() {
                            ""
                        } else {
                            " and"
                        };
                        writeln!(&mut output, "    ({}){}", assertion.text, suffix).unwrap();
                    }
                    writeln!(&mut output, "  )").unwrap();
                }
            }
        }

        writeln!(&mut output, "}}").unwrap();
        writeln!(&mut output, "run {} for 6", pred_name).unwrap();
    }

    output
}

fn find_alloy_assertions<'a>(
    problem: &'a Problem,
    set_name: &str,
    scope: AssertionScope,
) -> Vec<&'a Assertion> {
    let Some(set) = problem
        .assertion_sets
        .iter()
        .find(|set| set.name == set_name && set.scope == scope)
    else {
        return Vec::new();
    };

    set.assertions
        .iter()
        .filter(|assertion| {
            assertion
                .language
                .as_deref()
                .map(|language| language.eq_ignore_ascii_case("Alloy"))
                .unwrap_or(false)
        })
        .collect()
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
    let mut output = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }

    let mut output = if output.is_empty() {
        "Generated".to_string()
    } else {
        let mut collapsed = String::new();
        let mut prev_underscore = false;
        for ch in output.chars() {
            if ch == '_' {
                if prev_underscore {
                    continue;
                }
                prev_underscore = true;
            } else {
                prev_underscore = false;
            }
            collapsed.push(ch);
        }
        collapsed.trim_matches('_').to_string()
    };

    if let Some(first) = output.chars().next() {
        if first.is_ascii_digit() {
            output.insert(0, 'R');
        } else if first.is_ascii_lowercase() {
            if let Some(upper) = first.to_uppercase().next() {
                output.replace_range(0..first.len_utf8(), &upper.to_string());
            }
        }
    } else {
        output = "Generated".to_string();
    }

    if is_alloy_keyword(&output) || is_alloy_keyword(&output.to_ascii_lowercase()) {
        output.push('_');
    }

    output
}

fn is_alloy_keyword(name: &str) -> bool {
    matches!(
        name,
        "abstract"
            | "all"
            | "and"
            | "as"
            | "assert"
            | "at"
            | "but"
            | "check"
            | "disj"
            | "else"
            | "event"
            | "exactly"
            | "iff"
            | "implies"
            | "in"
            | "let"
            | "module"
            | "no"
            | "not"
            | "none"
            | "one"
            | "or"
            | "run"
            | "set"
            | "sig"
            | "some"
            | "sum"
            | "this"
            | "univ"
            | "Int"
            | "Boolean"
            | "seq"
            | "String"
            | "fact"
            | "pred"
            | "fun"
            | "assertion"
            | "enum"
            | "open"
            | "private"
            | "protected"
            | "public"
            | "extends"
            | "id"
    )
}

#[cfg(test)]
mod tests {
    use super::{generate_alloy, sanitize_name};
    use crate::ast::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn domain(name: &str, kind: DomainKind, role: DomainRole) -> Domain {
        Domain {
            name: name.to_string(),
            kind,
            role,
            marks: vec![],
            span: span(),
            source_path: None,
        }
    }

    #[test]
    fn sanitize_name_maps_invalid_identifiers() {
        assert_eq!(sanitize_name("1value"), "R1value");
        assert_eq!(sanitize_name("my domain"), "My_domain");
        assert_eq!(sanitize_name(""), "Generated");
        assert_eq!(sanitize_name("sig"), "Sig_");
    }

    #[test]
    fn emits_alloy_with_sanitized_names() {
        let problem = Problem {
            name: "1 bad thermostat".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("my domain", DomainKind::Causal, DomainRole::Machine),
                domain("sig", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "req".to_string(),
                frame: FrameType::RequiredBehavior,
                phenomena: vec![],
                marks: vec![],
                constraint: "ok".to_string(),
                constrains: None,
                reference: None,
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![
                AssertionSet {
                    name: "S".to_string(),
                    scope: AssertionScope::Specification,
                    assertions: vec![],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "W".to_string(),
                    scope: AssertionScope::WorldProperties,
                    assertions: vec![],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "R".to_string(),
                    scope: AssertionScope::RequirementAssertions,
                    assertions: vec![],
                    span: span(),
                    source_path: None,
                },
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "oblig".to_string(),
                specification_set: "S".to_string(),
                world_set: "W".to_string(),
                requirement_set: "R".to_string(),
                specification_ref: Reference {
                    name: "S".to_string(),
                    span: span(),
                },
                world_ref: Reference {
                    name: "W".to_string(),
                    span: span(),
                },
                requirement_ref: Reference {
                    name: "R".to_string(),
                    span: span(),
                },
                span: span(),
                source_path: None,
            }],
        };

        let output = generate_alloy(&problem);
        assert!(output.contains("module R1_bad_thermostat"));
        assert!(output.contains("one sig My_domain extends Domain {}"));
        assert!(output.contains("one sig Sig_ extends Domain {}"));
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
                marks: vec![],
                constraint: "maintain room".to_string(),
                constrains: Some(reference("Room")),
                reference: None,
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
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
                specification_ref: reference("S_control"),
                world_ref: reference("W_base"),
                requirement_ref: reference("R_goal"),
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

    #[test]
    fn emits_counterexample_formula_from_alloy_assertions() {
        let problem = Problem {
            name: "Adequacy".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Device", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![
                AssertionSet {
                    name: "S_control".to_string(),
                    scope: AssertionScope::Specification,
                    assertions: vec![Assertion {
                        text: "some Machine".to_string(),
                        language: Some("Alloy".to_string()),
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "W_base".to_string(),
                    scope: AssertionScope::WorldProperties,
                    assertions: vec![Assertion {
                        text: "some Device".to_string(),
                        language: Some("Alloy".to_string()),
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "R_goal".to_string(),
                    scope: AssertionScope::RequirementAssertions,
                    assertions: vec![Assertion {
                        text: "some Device".to_string(),
                        language: Some("Alloy".to_string()),
                        span: span(),
                    }],
                    span: span(),
                    source_path: None,
                },
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A_exec".to_string(),
                specification_set: "S_control".to_string(),
                world_set: "W_base".to_string(),
                requirement_set: "R_goal".to_string(),
                specification_ref: reference("S_control"),
                world_ref: reference("W_base"),
                requirement_ref: reference("R_goal"),
                span: span(),
                source_path: None,
            }],
        };

        let alloy = generate_alloy(&problem);
        assert!(alloy.contains("(some Machine)"));
        assert!(alloy.contains("(some Device)"));
        assert!(alloy.contains("not (some Device)"));
    }
}
