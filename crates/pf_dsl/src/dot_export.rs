use crate::ast::*;
use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Clone, Copy)]
enum DotView {
    Context,
    Problem,
    Decomposition,
}

pub fn to_dot(problem: &Problem) -> String {
    to_problem_dot(problem)
}

pub fn to_context_dot(problem: &Problem) -> String {
    build_dot(problem, DotView::Context)
}

pub fn to_problem_dot(problem: &Problem) -> String {
    build_dot(problem, DotView::Problem)
}

pub fn to_decomposition_dot(problem: &Problem) -> String {
    build_dot(problem, DotView::Decomposition)
}

fn build_dot(problem: &Problem, view: DotView) -> String {
    let mut dot = String::new();
    writeln!(
        &mut dot,
        "digraph \"{}\" {{",
        escape_dot_string(&problem.name)
    )
    .unwrap();
    writeln!(&mut dot, "    rankdir=LR;").unwrap();
    writeln!(
        &mut dot,
        "    node [shape=box, style=filled, fillcolor=white];"
    )
    .unwrap();

    write_domain_nodes(problem, &mut dot);

    match view {
        DotView::Context => {
            write_interface_edges(problem, &mut dot);
        }
        DotView::Problem => {
            write_requirement_nodes(problem, &mut dot, true);
            write_interface_edges(problem, &mut dot);
        }
        DotView::Decomposition => {
            write_requirement_nodes(problem, &mut dot, false);
            write_subproblem_nodes(problem, &mut dot);
        }
    }

    writeln!(&mut dot, "}}").unwrap();
    dot
}

fn write_domain_nodes(problem: &Problem, dot: &mut String) {
    for domain in &problem.domains {
        let (shape, color) = match domain.kind {
            DomainKind::Causal => ("box", "white"),
            DomainKind::Biddable => ("ellipse", "white"),
            DomainKind::Lexical => ("parallelogram", "white"),
            _ => ("box", "red"),
        };
        let (shape, color) = if domain.role == DomainRole::Machine {
            ("doublebox", "lightgrey")
        } else {
            (shape, color)
        };
        let label = format!("{} <<{:?}/{:?}>>", domain.name, domain.kind, domain.role);
        writeln!(
            dot,
            "    \"{}\" [label=\"{}\", shape={}, fillcolor={}];",
            escape_dot_string(&domain.name),
            escape_dot_string(&label),
            shape,
            color
        )
        .unwrap();
    }
}

fn write_requirement_nodes(problem: &Problem, dot: &mut String, include_domain_links: bool) {
    for req in &problem.requirements {
        let frame_label = match &req.frame {
            FrameType::RequiredBehavior => "RequiredBehavior",
            FrameType::CommandedBehavior => "CommandedBehavior",
            FrameType::InformationDisplay => "InformationDisplay",
            FrameType::SimpleWorkpieces => "SimpleWorkpieces",
            FrameType::Transformation => "Transformation",
            FrameType::Custom(s) => s.as_str(),
        };
        writeln!(
            dot,
            "    \"{}\" [shape=note, style=dashed, label=\"{}\"];",
            escape_dot_string(&req.name),
            escape_dot_string(&format!("{}\\n[{}]", req.name, frame_label)),
        )
        .unwrap();

        if !include_domain_links {
            continue;
        }

        if let Some(ref c) = req.constrains {
            writeln!(
                dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowhead=none, label=\"constrains\"];",
                escape_dot_string(&req.name),
                escape_dot_string(&c.name)
            )
            .unwrap();
        }
        if let Some(ref r) = req.reference {
            writeln!(
                dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowhead=none, label=\"references\"];",
                escape_dot_string(&req.name),
                escape_dot_string(&r.name)
            )
            .unwrap();
        }
    }
}

fn write_interface_edges(problem: &Problem, dot: &mut String) {
    // Aggregate all phenomena by unordered domain pair so we do not lose connections
    // when one interface contains multiple pairs.
    let mut edges: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for interface in &problem.interfaces {
        for phen in &interface.shared_phenomena {
            let pair = (phen.from.name.clone(), phen.to.name.clone());
            let key = if pair.0 <= pair.1 {
                pair
            } else {
                (pair.1, pair.0)
            };
            let symbol = match phen.type_ {
                PhenomenonType::Event => "E",
                PhenomenonType::Command => "C",
                PhenomenonType::State => "S",
                PhenomenonType::Value => "V",
            };
            edges.entry(key).or_default().push(format!(
                "{} -> {}: {} [{}]",
                phen.from.name, phen.to.name, phen.name, symbol
            ));
        }
    }

    for ((src, dst), labels) in edges {
        let label_str = labels.join("\\n");
        writeln!(
            dot,
            "    \"{}\" -> \"{}\" [dir=both, label=\"{}\"];",
            escape_dot_string(&src),
            escape_dot_string(&dst),
            escape_dot_string(&label_str)
        )
        .unwrap();
    }
}

fn write_subproblem_nodes(problem: &Problem, dot: &mut String) {
    for subproblem in &problem.subproblems {
        writeln!(
            dot,
            "    \"subproblem:{}\" [shape=folder, fillcolor=lightyellow, label=\"{}\"];",
            escape_dot_string(&subproblem.name),
            escape_dot_string(&format!("subproblem\\n{}", subproblem.name))
        )
        .unwrap();

        if let Some(machine) = &subproblem.machine {
            writeln!(
                dot,
                "    \"subproblem:{}\" -> \"{}\" [label=\"machine\", style=bold];",
                escape_dot_string(&subproblem.name),
                escape_dot_string(&machine.name)
            )
            .unwrap();
        }

        for participant in &subproblem.participants {
            writeln!(
                dot,
                "    \"subproblem:{}\" -> \"{}\" [label=\"participant\", style=dotted];",
                escape_dot_string(&subproblem.name),
                escape_dot_string(&participant.name)
            )
            .unwrap();
        }

        for requirement in &subproblem.requirements {
            writeln!(
                dot,
                "    \"subproblem:{}\" -> \"{}\" [label=\"includes\", style=dashed];",
                escape_dot_string(&subproblem.name),
                escape_dot_string(&requirement.name)
            )
            .unwrap();
        }
    }
}

fn escape_dot_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len() + 4);
    for ch in input.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{to_context_dot, to_decomposition_dot, to_dot, to_problem_dot};
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
    fn keeps_all_pairs_from_interface_phenomena() {
        let problem = Problem {
            name: "P".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("A", DomainKind::Causal, DomainRole::Machine),
                domain("B", DomainKind::Causal, DomainRole::Given),
                domain("C", DomainKind::Causal, DomainRole::Given),
                domain("D", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![Interface {
                name: "mixed".to_string(),
                connects: vec![
                    reference("A"),
                    reference("B"),
                    reference("C"),
                    reference("D"),
                ],
                shared_phenomena: vec![
                    Phenomenon {
                        name: "e1".to_string(),
                        type_: PhenomenonType::Event,
                        from: reference("A"),
                        to: reference("B"),
                        controlled_by: reference("A"),
                        span: span(),
                    },
                    Phenomenon {
                        name: "e2".to_string(),
                        type_: PhenomenonType::Event,
                        from: reference("C"),
                        to: reference("D"),
                        controlled_by: reference("C"),
                        span: span(),
                    },
                ],
                span: span(),
                source_path: None,
            }],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let dot = to_dot(&problem);
        assert!(dot.contains("\"A\" -> \"B\""));
        assert!(dot.contains("\"C\" -> \"D\""));
    }

    #[test]
    fn context_view_excludes_requirement_nodes() {
        let problem = Problem {
            name: "ContextOnly".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Sensor", DomainKind::Causal, DomainRole::Given),
            ],
            interfaces: vec![Interface {
                name: "M-S".to_string(),
                connects: vec![reference("Machine"), reference("Sensor")],
                shared_phenomena: vec![Phenomenon {
                    name: "Observe".to_string(),
                    type_: PhenomenonType::Event,
                    from: reference("Sensor"),
                    to: reference("Machine"),
                    controlled_by: reference("Sensor"),
                    span: span(),
                }],
                span: span(),
                source_path: None,
            }],
            requirements: vec![Requirement {
                name: "ReqA".to_string(),
                frame: FrameType::RequiredBehavior,
                constrains: Some(reference("Sensor")),
                reference: None,
                phenomena: vec![],
                constraint: String::new(),
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let dot = to_context_dot(&problem);
        assert!(dot.contains("[dir=both, label=\""));
        assert!(dot.contains("Observe [E]"));
        assert!(!dot.contains("ReqA\\n[RequiredBehavior]"));
        assert!(!dot.contains("label=\"constrains\""));
    }

    #[test]
    fn problem_view_includes_requirement_links() {
        let problem = Problem {
            name: "ProblemView".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Operator", DomainKind::Biddable, DomainRole::Given),
            ],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "ReqB".to_string(),
                frame: FrameType::CommandedBehavior,
                constrains: Some(reference("Machine")),
                reference: Some(reference("Operator")),
                phenomena: vec![],
                constraint: String::new(),
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let dot = to_problem_dot(&problem);
        assert!(dot.contains("\"ReqB\" [shape=note"));
        assert!(dot.contains("label=\"constrains\""));
        assert!(dot.contains("label=\"references\""));
    }

    #[test]
    fn decomposition_view_includes_subproblems_and_excludes_interface_edges() {
        let problem = Problem {
            name: "DecompositionView".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                domain("Machine", DomainKind::Causal, DomainRole::Machine),
                domain("Ledger", DomainKind::Lexical, DomainRole::Given),
            ],
            interfaces: vec![Interface {
                name: "M-L".to_string(),
                connects: vec![reference("Machine"), reference("Ledger")],
                shared_phenomena: vec![Phenomenon {
                    name: "Persist".to_string(),
                    type_: PhenomenonType::Value,
                    from: reference("Machine"),
                    to: reference("Ledger"),
                    controlled_by: reference("Machine"),
                    span: span(),
                }],
                span: span(),
                source_path: None,
            }],
            requirements: vec![Requirement {
                name: "ReqStore".to_string(),
                frame: FrameType::Transformation,
                constrains: Some(reference("Ledger")),
                reference: None,
                phenomena: vec![],
                constraint: String::new(),
                span: span(),
                source_path: None,
            }],
            subproblems: vec![Subproblem {
                name: "StorageFlow".to_string(),
                machine: Some(reference("Machine")),
                participants: vec![reference("Machine"), reference("Ledger")],
                requirements: vec![reference("ReqStore")],
                span: span(),
                source_path: None,
            }],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let dot = to_decomposition_dot(&problem);
        assert!(dot.contains("\"subproblem:StorageFlow\""));
        assert!(dot.contains("label=\"machine\""));
        assert!(dot.contains("label=\"includes\""));
        assert!(!dot.contains("[dir=both"));
    }

    #[test]
    fn escapes_dot_string_special_characters() {
        assert_eq!(
            super::escape_dot_string("a\"b\\c\nd\te\rf"),
            r#"a\"b\\c\nd\te\rf"#
        );
    }

    #[test]
    fn to_dot_escapes_domain_ids_and_labels() {
        let problem = Problem {
            name: "P".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![domain(r#"D"1"#, DomainKind::Causal, DomainRole::Given)],
            interfaces: vec![],
            requirements: vec![Requirement {
                name: "Req\n1".to_string(),
                frame: FrameType::RequiredBehavior,
                constrains: None,
                reference: None,
                phenomena: vec![],
                constraint: String::new(),
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![],
            correctness_arguments: vec![],
        };

        let dot = to_dot(&problem);
        let escaped_domain_id = super::escape_dot_string("D\"1");
        let escaped_domain_label = super::escape_dot_string("D\"1 <<Causal/Given>>");
        let escaped_req_label =
            super::escape_dot_string(&format!("{}\\n[{}]", "Req\n1", "RequiredBehavior"));

        assert!(dot.contains(&format!("\"{}\"", escaped_domain_id)));
        assert!(dot.contains(&format!(r#"label="{}","#, escaped_domain_label)));
        assert!(dot.contains(&format!(r#"label="{}""#, escaped_req_label)));
        assert!(dot.contains(r#"digraph "P" {"#));
    }
}
