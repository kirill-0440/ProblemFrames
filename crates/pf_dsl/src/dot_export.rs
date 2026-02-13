use crate::ast::*;
use std::collections::BTreeMap;
use std::fmt::Write;

pub fn to_dot(problem: &Problem) -> String {
    let mut dot = String::new();
    writeln!(&mut dot, "digraph \"{}\" {{", problem.name).unwrap();
    writeln!(&mut dot, "    rankdir=LR;").unwrap();
    writeln!(
        &mut dot,
        "    node [shape=box, style=filled, fillcolor=white];"
    )
    .unwrap();

    // 1. Nodes (Domains)
    for domain in &problem.domains {
        let (shape, color) = match domain.kind {
            DomainKind::Causal => ("box", "white"),
            DomainKind::Biddable => ("ellipse", "white"), // People as ellipses or ovals
            DomainKind::Lexical => ("parallelogram", "white"), // Data
            _ => ("box", "red"),
        };
        let (shape, color) = if domain.role == DomainRole::Machine {
            ("doublebox", "lightgrey")
        } else {
            (shape, color)
        };
        let label = format!("{} <<{:?}/{:?}>>", domain.name, domain.kind, domain.role);
        writeln!(
            &mut dot,
            "    \"{}\" [label=\"{}\", shape={}, fillcolor={}];",
            domain.name, label, shape, color
        )
        .unwrap();
    }

    // 2. Requirements (Ovals)
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
            &mut dot,
            "    \"{}\" [shape=note, style=dashed, label=\"{}\\n[{}]\"];",
            req.name, req.name, frame_label
        )
        .unwrap();

        // Connect Requirement to Constrained/Referenced domains
        if let Some(ref c) = req.constrains {
            writeln!(
                &mut dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowhead=none, label=\"constrains\"];",
                req.name, c.name
            )
            .unwrap();
        }
        if let Some(ref r) = req.reference {
            writeln!(
                &mut dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowhead=none, label=\"references\"];",
                req.name, r.name
            )
            .unwrap();
        }
    }

    // 3. Edges (Interfaces)
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
            &mut dot,
            "    \"{}\" -> \"{}\" [dir=both, label=\"{}\"];",
            src, dst, label_str
        )
        .unwrap();
    }

    writeln!(&mut dot, "}}").unwrap();
    dot
}

#[cfg(test)]
mod tests {
    use super::to_dot;
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
}
