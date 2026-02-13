use crate::ast::*;
use crate::decomposition_closure::{
    analyze_decomposition_closure, render_decomposition_closure_section,
};

pub fn generate_report(problem: &Problem) -> String {
    let mut report = String::new();

    report.push_str(&format!("# Problem Report: {}\n\n", problem.name));

    report.push_str("## 1. Domains\n");
    for d in &problem.domains {
        report.push_str(&format!("- **{}** ({:?}/{:?})\n", d.name, d.kind, d.role));
    }
    report.push('\n');

    report.push_str("## 2. Interfaces\n");
    for i in &problem.interfaces {
        report.push_str(&format!("- **Interface**: {}\n", i.name));
        for p in &i.shared_phenomena {
            let symbol = match p.type_ {
                PhenomenonType::Event => "Event",
                PhenomenonType::Command => "Command",
                PhenomenonType::State => "State",
                PhenomenonType::Value => "Value",
            };
            report.push_str(&format!(
                "  - [{}] {} ({} -> {})\n",
                symbol, p.name, p.from.name, p.to.name
            ));
        }
    }
    report.push('\n');

    report.push_str("## 3. Requirements\n");
    for r in &problem.requirements {
        report.push_str(&format!("### {}\n", r.name));
        report.push_str(&format!("- **Frame**: {:?}\n", r.frame));
        report.push_str(&format!("- **Constraint**: {}\n", r.constraint));
        if let Some(ref c) = r.constrains {
            report.push_str(&format!("- **Constrains**: {}\n", c.name));
        }
        if let Some(ref rf) = r.reference {
            report.push_str(&format!("- **Reference**: {}\n", rf.name));
        }
        report.push('\n');
    }

    if !problem.subproblems.is_empty() {
        report.push_str("## 4. Subproblems\n");
        for subproblem in &problem.subproblems {
            report.push_str(&format!("### {}\n", subproblem.name));
            if let Some(machine) = &subproblem.machine {
                report.push_str(&format!("- **Machine**: {}\n", machine.name));
            }
            if !subproblem.participants.is_empty() {
                let participants = subproblem
                    .participants
                    .iter()
                    .map(|participant| participant.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                report.push_str(&format!("- **Participants**: {}\n", participants));
            }
            if !subproblem.requirements.is_empty() {
                let requirements = subproblem
                    .requirements
                    .iter()
                    .map(|requirement| requirement.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                report.push_str(&format!("- **Requirements**: {}\n", requirements));
            }
            report.push('\n');
        }
    }

    let closure = analyze_decomposition_closure(problem);
    report.push_str("## 5. Decomposition Closure\n");
    report.push_str(&render_decomposition_closure_section(&closure));
    report.push('\n');

    report
}
