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
        let (shape, color) = match domain.domain_type {
            DomainType::Machine => ("doublebox", "lightgrey"), // Machine often double striped
            DomainType::Causal => ("box", "white"),
            DomainType::Biddable => ("ellipse", "white"), // People as ellipses or ovals
            DomainType::Lexical => ("parallelogram", "white"), // Data
            _ => ("box", "red"),
        };
        // Jackson notation suggests specific decorations, but standard shapes approximate well for now.
        // Machine: Double vertical stripe (not standard in graphviz, using box for now)
        // Causal: Plain box
        // Biddable: Person icon (using ellipse)
        // Lexical: Data stripe (using parallelogram)

        let label = format!("{} <<{:?}>>", domain.name, domain.domain_type);
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

    fn domain(name: &str, domain_type: DomainType) -> Domain {
        Domain {
            name: name.to_string(),
            domain_type,
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
                domain("A", DomainType::Machine),
                domain("B", DomainType::Causal),
                domain("C", DomainType::Causal),
                domain("D", DomainType::Causal),
            ],
            interfaces: vec![Interface {
                name: "mixed".to_string(),
                shared_phenomena: vec![
                    Phenomenon {
                        name: "e1".to_string(),
                        type_: PhenomenonType::Event,
                        from: reference("A"),
                        to: reference("B"),
                        span: span(),
                    },
                    Phenomenon {
                        name: "e2".to_string(),
                        type_: PhenomenonType::Event,
                        from: reference("C"),
                        to: reference("D"),
                        span: span(),
                    },
                ],
                span: span(),
                source_path: None,
            }],
            requirements: vec![],
        };

        let dot = to_dot(&problem);
        assert!(dot.contains("\"A\" -> \"B\""));
        assert!(dot.contains("\"C\" -> \"D\""));
    }
}
