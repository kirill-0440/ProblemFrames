use crate::ast::{
    AssertionScope, CorrectnessArgument, Domain, DomainKind, DomainRole, FrameType, Interface,
    Phenomenon, PhenomenonType, Problem, Requirement,
};

fn escape_lean_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
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

fn domain_kind_expr(kind: &DomainKind) -> String {
    match kind {
        DomainKind::Biddable => "DomainKind.biddable".to_string(),
        DomainKind::Causal => "DomainKind.causal".to_string(),
        DomainKind::Lexical => "DomainKind.lexical".to_string(),
        DomainKind::Unknown(label) => {
            format!("DomainKind.unknown \"{}\"", escape_lean_string(label))
        }
    }
}

fn domain_role_expr(role: &DomainRole) -> String {
    match role {
        DomainRole::Given => "DomainRole.given".to_string(),
        DomainRole::Designed => "DomainRole.designed".to_string(),
        DomainRole::Machine => "DomainRole.machine".to_string(),
        DomainRole::Unknown(label) => {
            format!("DomainRole.unknown \"{}\"", escape_lean_string(label))
        }
    }
}

fn phenomenon_type_expr(kind: &PhenomenonType) -> &'static str {
    match kind {
        PhenomenonType::Event => "PhenomenonType.event",
        PhenomenonType::Command => "PhenomenonType.command",
        PhenomenonType::State => "PhenomenonType.state",
        PhenomenonType::Value => "PhenomenonType.value",
    }
}

fn frame_type_expr(frame: &FrameType) -> String {
    match frame {
        FrameType::RequiredBehavior => "FrameType.requiredBehavior".to_string(),
        FrameType::CommandedBehavior => "FrameType.commandedBehavior".to_string(),
        FrameType::InformationDisplay => "FrameType.informationDisplay".to_string(),
        FrameType::SimpleWorkpieces => "FrameType.simpleWorkpieces".to_string(),
        FrameType::Transformation => "FrameType.transformation".to_string(),
        FrameType::Custom(label) => format!("FrameType.custom \"{}\"", escape_lean_string(label)),
    }
}

fn option_string_expr(value: Option<&str>) -> String {
    match value {
        Some(v) => format!("some \"{}\"", escape_lean_string(v)),
        None => "none".to_string(),
    }
}

fn assertion_scope_expr(scope: &AssertionScope) -> &'static str {
    match scope {
        AssertionScope::WorldProperties => "AssertionScope.worldProperties",
        AssertionScope::Specification => "AssertionScope.specification",
        AssertionScope::RequirementAssertions => "AssertionScope.requirementAssertions",
    }
}

fn emit_domains(domains: &[Domain], output: &mut String) {
    let mut sorted = domains.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("def domains : List Domain := [\n");
    for domain in sorted {
        output.push_str(&format!(
            "  {{ name := \"{}\", kind := {}, role := {} }},\n",
            escape_lean_string(&domain.name),
            domain_kind_expr(&domain.kind),
            domain_role_expr(&domain.role),
        ));
    }
    output.push_str("]\n\n");
}

fn emit_phenomena(phenomena: &[Phenomenon], output: &mut String) {
    let mut sorted = phenomena.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("[\n");
    for phenomenon in sorted {
        output.push_str(&format!(
            "      {{ name := \"{}\", kind := {}, fromDomain := \"{}\", toDomain := \"{}\", controlledBy := \"{}\" }},\n",
            escape_lean_string(&phenomenon.name),
            phenomenon_type_expr(&phenomenon.type_),
            escape_lean_string(&phenomenon.from.name),
            escape_lean_string(&phenomenon.to.name),
            escape_lean_string(&phenomenon.controlled_by.name),
        ));
    }
    output.push_str("    ]");
}

fn emit_interfaces(interfaces: &[Interface], output: &mut String) {
    let mut sorted = interfaces.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("def interfaces : List Interface := [\n");
    for interface in sorted {
        output.push_str(&format!(
            "  {{ name := \"{}\", connects := [{}], phenomena := ",
            escape_lean_string(&interface.name),
            interface
                .connects
                .iter()
                .map(|reference| format!("\"{}\"", escape_lean_string(&reference.name)))
                .collect::<Vec<_>>()
                .join(", "),
        ));
        emit_phenomena(&interface.shared_phenomena, output);
        output.push_str(" },\n");
    }
    output.push_str("]\n\n");
}

fn emit_requirements(requirements: &[Requirement], output: &mut String) {
    let mut sorted = requirements.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("def requirements : List Requirement := [\n");
    for requirement in sorted {
        output.push_str(&format!(
            "  {{ name := \"{}\", frame := {}, constraint := \"{}\", constrains := {}, reference := {} }},\n",
            escape_lean_string(&requirement.name),
            frame_type_expr(&requirement.frame),
            escape_lean_string(&requirement.constraint),
            option_string_expr(requirement.constrains.as_ref().map(|reference| reference.name.as_str())),
            option_string_expr(requirement.reference.as_ref().map(|reference| reference.name.as_str())),
        ));
    }
    output.push_str("]\n\n");
}

fn emit_assertion_sets(problem: &Problem, output: &mut String) {
    let mut sorted = problem.assertion_sets.clone();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("def assertionSets : List AssertionSet := [\n");
    for assertion_set in sorted {
        let assertions = assertion_set
            .assertions
            .iter()
            .map(|assertion| format!("\"{}\"", escape_lean_string(&assertion.text)))
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "  {{ name := \"{}\", scope := {}, assertions := [{}] }},\n",
            escape_lean_string(&assertion_set.name),
            assertion_scope_expr(&assertion_set.scope),
            assertions,
        ));
    }
    output.push_str("]\n\n");
}

fn emit_correctness_arguments(arguments: &[CorrectnessArgument], output: &mut String) {
    let mut sorted = arguments.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));

    output.push_str("def correctnessArguments : List CorrectnessArgument := [\n");
    for argument in sorted {
        output.push_str(&format!(
            "  {{ name := \"{}\", specificationSet := \"{}\", worldSet := \"{}\", requirementSet := \"{}\" }},\n",
            escape_lean_string(&argument.name),
            escape_lean_string(&argument.specification_set),
            escape_lean_string(&argument.world_set),
            escape_lean_string(&argument.requirement_set),
        ));
    }
    output.push_str("]\n\n");
}

pub fn generate_lean_model(problem: &Problem) -> String {
    let mut output = String::new();

    output.push_str("-- Auto-generated by `pf_dsl --lean-model`.\n");
    output.push_str("-- This artifact is part of the non-blocking formal research track.\n\n");
    output.push_str("set_option autoImplicit false\n\n");
    output.push_str("namespace ProblemFramesGenerated\n\n");
    output.push_str("inductive DomainKind where\n");
    output.push_str("  | biddable\n");
    output.push_str("  | causal\n");
    output.push_str("  | lexical\n");
    output.push_str("  | unknown (label : String)\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("inductive DomainRole where\n");
    output.push_str("  | given\n");
    output.push_str("  | designed\n");
    output.push_str("  | machine\n");
    output.push_str("  | unknown (label : String)\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("inductive PhenomenonType where\n");
    output.push_str("  | event\n");
    output.push_str("  | command\n");
    output.push_str("  | state\n");
    output.push_str("  | value\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("inductive FrameType where\n");
    output.push_str("  | requiredBehavior\n");
    output.push_str("  | commandedBehavior\n");
    output.push_str("  | informationDisplay\n");
    output.push_str("  | simpleWorkpieces\n");
    output.push_str("  | transformation\n");
    output.push_str("  | custom (label : String)\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("inductive AssertionScope where\n");
    output.push_str("  | worldProperties\n");
    output.push_str("  | specification\n");
    output.push_str("  | requirementAssertions\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure Domain where\n");
    output.push_str("  name : String\n");
    output.push_str("  kind : DomainKind\n");
    output.push_str("  role : DomainRole\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure Phenomenon where\n");
    output.push_str("  name : String\n");
    output.push_str("  kind : PhenomenonType\n");
    output.push_str("  fromDomain : String\n");
    output.push_str("  toDomain : String\n");
    output.push_str("  controlledBy : String\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure Interface where\n");
    output.push_str("  name : String\n");
    output.push_str("  connects : List String\n");
    output.push_str("  phenomena : List Phenomenon\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure Requirement where\n");
    output.push_str("  name : String\n");
    output.push_str("  frame : FrameType\n");
    output.push_str("  constraint : String\n");
    output.push_str("  constrains : Option String\n");
    output.push_str("  reference : Option String\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure AssertionSet where\n");
    output.push_str("  name : String\n");
    output.push_str("  scope : AssertionScope\n");
    output.push_str("  assertions : List String\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str("structure CorrectnessArgument where\n");
    output.push_str("  name : String\n");
    output.push_str("  specificationSet : String\n");
    output.push_str("  worldSet : String\n");
    output.push_str("  requirementSet : String\n");
    output.push_str("  deriving Repr, DecidableEq\n\n");
    output.push_str(&format!(
        "def problemName : String := \"{}\"\n\n",
        escape_lean_string(&problem.name)
    ));

    emit_domains(&problem.domains, &mut output);
    emit_interfaces(&problem.interfaces, &mut output);
    emit_requirements(&problem.requirements, &mut output);
    emit_assertion_sets(problem, &mut output);
    emit_correctness_arguments(&problem.correctness_arguments, &mut output);

    output.push_str("/-- Placeholder theorem for the Lean research track. -/\n");
    output.push_str("theorem machineBoundaryCaptured : Prop := by\n");
    output.push_str("  sorry\n\n");
    output.push_str("/-- Placeholder theorem for interface/controller consistency proof. -/\n");
    output.push_str("theorem interfaceControllersDeclared : Prop := by\n");
    output.push_str("  sorry\n\n");
    output.push_str("/-- Placeholder theorem for W/S/R argument closure proof. -/\n");
    output.push_str("theorem correctnessArgumentsStructured : Prop := by\n");
    output.push_str("  sorry\n\n");
    output.push_str("end ProblemFramesGenerated\n");

    output
}

#[cfg(test)]
mod tests {
    use super::generate_lean_model;
    use crate::ast::{
        Assertion, AssertionScope, AssertionSet, CorrectnessArgument, Domain, DomainKind,
        DomainRole, FrameType, Interface, Phenomenon, PhenomenonType, Problem, Reference, Span,
    };

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn reference(name: &str) -> Reference {
        Reference {
            name: name.to_string(),
            span: span(),
        }
    }

    #[test]
    fn generates_deterministic_lean_model_output() {
        let problem = Problem {
            name: "LeanExport".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![
                Domain {
                    name: "Controller".to_string(),
                    kind: DomainKind::Causal,
                    role: DomainRole::Machine,
                    marks: vec![],
                    span: span(),
                    source_path: None,
                },
                Domain {
                    name: "Operator".to_string(),
                    kind: DomainKind::Biddable,
                    role: DomainRole::Given,
                    marks: vec![],
                    span: span(),
                    source_path: None,
                },
            ],
            interfaces: vec![Interface {
                name: "Operator-Controller".to_string(),
                connects: vec![reference("Operator"), reference("Controller")],
                shared_phenomena: vec![Phenomenon {
                    name: "Command".to_string(),
                    type_: PhenomenonType::Command,
                    from: reference("Operator"),
                    to: reference("Controller"),
                    controlled_by: reference("Operator"),
                    span: span(),
                }],
                span: span(),
                source_path: None,
            }],
            requirements: vec![crate::ast::Requirement {
                name: "SafeOperation".to_string(),
                frame: FrameType::CommandedBehavior,
                phenomena: vec![],
                marks: vec![],
                constraint: "operate safely".to_string(),
                constrains: Some(reference("Controller")),
                reference: Some(reference("Operator")),
                span: span(),
                source_path: None,
            }],
            subproblems: vec![],
            assertion_sets: vec![AssertionSet {
                name: "S_main".to_string(),
                scope: AssertionScope::Specification,
                assertions: vec![Assertion {
                    text: "command implies action".to_string(),
                    language: Some("LTL".to_string()),
                    span: span(),
                }],
                span: span(),
                source_path: None,
            }],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A1".to_string(),
                specification_set: "S_main".to_string(),
                world_set: "W_env".to_string(),
                requirement_set: "R_goal".to_string(),
                specification_ref: reference("S_main"),
                world_ref: reference("W_env"),
                requirement_ref: reference("R_goal"),
                span: span(),
                source_path: None,
            }],
        };

        let first = generate_lean_model(&problem);
        let second = generate_lean_model(&problem);

        assert_eq!(first, second);
        assert!(first.contains("def problemName : String := \"LeanExport\""));
        assert!(first.contains("def domains : List Domain"));
        assert!(first.contains("def interfaces : List Interface"));
        assert!(first.contains("theorem machineBoundaryCaptured"));
    }
}
