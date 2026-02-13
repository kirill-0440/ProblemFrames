use crate::ast::*;

pub fn generate_report(problem: &Problem) -> String {
    let mut report = String::new();

    report.push_str(&format!("# Problem Report: {}\n\n", problem.name));

    report.push_str("## 1. Domains\n");
    for d in &problem.domains {
        report.push_str(&format!("- **{}** ({:?}/{:?})\n", d.name, d.kind, d.role));
    }
    report.push('\n');

    report.push_str("## 2. Intefaces\n");
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

    report
}
