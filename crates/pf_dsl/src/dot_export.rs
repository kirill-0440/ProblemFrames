use crate::ast::*;
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
        if !req.constrains.is_empty() {
            writeln!(
                &mut dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowheaders=none, label=\"constrains\"];",
                req.name, req.constrains
            )
            .unwrap();
        }
        if !req.reference.is_empty() {
            writeln!(
                &mut dot,
                "    \"{}\" -> \"{}\" [style=dashed, arrowheaders=none, label=\"references\"];",
                req.name, req.reference
            )
            .unwrap();
        }
    }

    // 3. Edges (Interfaces)
    // We need to aggregate phenomena between pairs of domains to draw single edges with multiple labels
    // For now, simpler approach: draw edges for each interface
    // Ideally, we group by (A, B) pair.

    // Simplification: Iterate interfaces, determine connected domains from phenomena.
    // If an interface explicitly named "A-B", we might guess, but better to look at phenomena.

    for interface in &problem.interfaces {
        // Find all unique (from, to) pairs in this interface
        // And construct a label list
        let mut connections: Vec<(String, String)> = vec![];
        let mut labels: Vec<String> = vec![];

        for phen in &interface.shared_phenomena {
            let pair = (phen.from.clone(), phen.to.clone());
            if !connections.contains(&pair)
                && !connections.contains(&(pair.1.clone(), pair.0.clone()))
            {
                connections.push(pair);
            }
            let symbol = match phen.type_ {
                PhenomenonType::Event => "E",
                PhenomenonType::Command => "C",
                PhenomenonType::State => "S",
                PhenomenonType::Value => "V",
            };
            labels.push(format!("{} [{}]", phen.name, symbol));
        }

        // Draw edge between the first identified pair (heuristic)
        if let Some((src, dst)) = connections.first() {
            let label_str = labels.join("\\n");
            writeln!(
                &mut dot,
                "    \"{}\" -> \"{}\" [dir=both, label=\"{}\"];",
                src, dst, label_str
            )
            .unwrap();
        }
    }

    writeln!(&mut dot, "}}").unwrap();
    dot
}
