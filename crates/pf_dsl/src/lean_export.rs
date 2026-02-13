use crate::ast::{
    AssertionScope, AssertionSet, CorrectnessArgument, Domain, DomainKind, DomainRole, FrameType,
    Interface, Phenomenon, PhenomenonType, Problem, Requirement,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

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

fn lean_ident(value: &str) -> String {
    let mut ident = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            ident.push(ch);
        } else {
            ident.push('_');
        }
    }

    if ident.is_empty() {
        ident.push_str("arg");
    }

    if ident.as_bytes()[0].is_ascii_digit() {
        ident.insert(0, '_');
    }

    ident
}

fn lean_string_list_expr(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{}\"", escape_lean_string(value)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn lean_atom_assertion_values(set: &AssertionSet) -> Vec<String> {
    set.assertions
        .iter()
        .filter(|assertion| assertion.language.as_deref() == Some("LeanAtom"))
        .map(|assertion| assertion.text.clone())
        .collect()
}

enum FormalCoverageDecision {
    Formalized {
        specification_values: Vec<String>,
        world_values: Vec<String>,
        requirement_values: Vec<String>,
    },
    Skipped {
        reason: &'static str,
    },
}

fn evaluate_formal_argument(
    argument: &CorrectnessArgument,
    sets_by_name: &BTreeMap<&str, &AssertionSet>,
) -> FormalCoverageDecision {
    let Some(specification_set) = sets_by_name.get(argument.specification_set.as_str()) else {
        return FormalCoverageDecision::Skipped {
            reason: "missing_specification_set",
        };
    };
    let Some(world_set) = sets_by_name.get(argument.world_set.as_str()) else {
        return FormalCoverageDecision::Skipped {
            reason: "missing_world_set",
        };
    };
    let Some(requirement_set) = sets_by_name.get(argument.requirement_set.as_str()) else {
        return FormalCoverageDecision::Skipped {
            reason: "missing_requirement_set",
        };
    };

    let specification_values = lean_atom_assertion_values(specification_set);
    let world_values = lean_atom_assertion_values(world_set);
    let requirement_values = lean_atom_assertion_values(requirement_set);

    // Mixed assertion sets are allowed; formal closure is computed over the
    // LeanAtom projection so narrative tracks (e.g. LTL) can coexist.
    if specification_values.is_empty() {
        return FormalCoverageDecision::Skipped {
            reason: "no_leanatom_specification_projection",
        };
    }
    if world_values.is_empty() {
        return FormalCoverageDecision::Skipped {
            reason: "no_leanatom_world_projection",
        };
    }
    if requirement_values.is_empty() {
        return FormalCoverageDecision::Skipped {
            reason: "no_leanatom_requirement_projection",
        };
    }

    // The current strict closure mode proves formal entailment only when
    // requirement LeanAtom projection mirrors specification LeanAtom
    // projection exactly.
    if specification_values != requirement_values {
        return FormalCoverageDecision::Skipped {
            reason: "requirement_not_mirror_specification_projection",
        };
    }

    FormalCoverageDecision::Formalized {
        specification_values,
        world_values,
        requirement_values,
    }
}

fn emit_formal_correctness_argument_proofs(problem: &Problem, output: &mut String) {
    let mut sets_by_name = BTreeMap::new();
    for set in &problem.assertion_sets {
        sets_by_name.insert(set.name.as_str(), set);
    }

    output.push_str("def Holds (sem : String -> Prop) (xs : List String) : Prop :=\n");
    output.push_str("  forall a, List.Mem a xs -> sem a\n\n");

    let mut sorted_arguments = problem.correctness_arguments.clone();
    sorted_arguments.sort_by(|left, right| left.name.cmp(&right.name));
    let mut emitted_names = BTreeSet::new();

    for argument in sorted_arguments {
        let (spec_values, world_values, req_values) =
            match evaluate_formal_argument(&argument, &sets_by_name) {
                FormalCoverageDecision::Formalized {
                    specification_values,
                    world_values,
                    requirement_values,
                } => (specification_values, world_values, requirement_values),
                FormalCoverageDecision::Skipped { .. } => continue,
            };

        let mut base = lean_ident(&argument.name);
        let mut suffix = 2usize;
        while !emitted_names.insert(base.clone()) {
            base = format!("{}_{}", lean_ident(&argument.name), suffix);
            suffix += 1;
        }

        output.push_str(&format!(
            "def {}SpecAssertions : List String := {}\n",
            base,
            lean_string_list_expr(&spec_values),
        ));
        output.push_str(&format!(
            "def {}WorldAssertions : List String := {}\n",
            base,
            lean_string_list_expr(&world_values),
        ));
        output.push_str(&format!(
            "def {}ReqAssertions : List String := {}\n\n",
            base,
            lean_string_list_expr(&req_values),
        ));

        output.push_str(&format!(
            "/-- Closed coverage witness generated for correctness argument `{}`. -/\n",
            escape_lean_string(&argument.name),
        ));
        output.push_str(&format!("theorem {}CoverageClosed :\n", base,));
        output.push_str(&format!(
            "    forall a, List.Mem a {}ReqAssertions -> List.Mem a {}SpecAssertions \\/ List.Mem a {}WorldAssertions := by\n",
            base, base, base,
        ));
        output.push_str("  intro a hReq\n");
        output.push_str(&format!(
            "  exact Or.inl (by simpa [{}ReqAssertions, {}SpecAssertions] using hReq)\n\n",
            base, base,
        ));
        output.push_str(&format!(
            "/-- Formal W/S/R entailment closure for correctness argument `{}`. -/\n",
            escape_lean_string(&argument.name),
        ));
        output.push_str(&format!(
            "theorem {}Entailment (sem : String -> Prop)\n",
            base
        ));
        output.push_str(&format!("    (hSpec : Holds sem {}SpecAssertions)\n", base));
        output.push_str(&format!(
            "    (hWorld : Holds sem {}WorldAssertions) :\n",
            base
        ));
        output.push_str(&format!("    Holds sem {}ReqAssertions := by\n", base));
        output.push_str("  intro a hReq\n");
        output.push_str(&format!(
            "  have hCov : List.Mem a {}SpecAssertions \\/ List.Mem a {}WorldAssertions :=\n",
            base, base,
        ));
        output.push_str(&format!("    {}CoverageClosed a hReq\n", base));
        output.push_str("  cases hCov with\n");
        output.push_str("  | inl hSpecMem => exact hSpec a hSpecMem\n");
        output.push_str("  | inr hWorldMem => exact hWorld a hWorldMem\n\n");
    }
}

#[derive(Serialize)]
struct LeanCoverageFormalizedEntry {
    argument: String,
    mode: String,
    specification_assertions: usize,
    world_assertions: usize,
    requirement_assertions: usize,
}

#[derive(Serialize)]
struct LeanCoverageSkippedEntry {
    argument: String,
    reason: String,
}

#[derive(Serialize)]
struct LeanCoverageReport {
    problem: String,
    total_correctness_arguments: usize,
    formalized_count: usize,
    skipped_count: usize,
    formalized: Vec<LeanCoverageFormalizedEntry>,
    skipped: Vec<LeanCoverageSkippedEntry>,
}

pub fn generate_lean_coverage_json(problem: &Problem) -> Result<String, serde_json::Error> {
    let mut sets_by_name = BTreeMap::new();
    for set in &problem.assertion_sets {
        sets_by_name.insert(set.name.as_str(), set);
    }

    let mut sorted_arguments = problem.correctness_arguments.clone();
    sorted_arguments.sort_by(|left, right| left.name.cmp(&right.name));

    let mut formalized = Vec::new();
    let mut skipped = Vec::new();

    for argument in sorted_arguments {
        match evaluate_formal_argument(&argument, &sets_by_name) {
            FormalCoverageDecision::Formalized {
                specification_values,
                world_values,
                requirement_values,
            } => {
                formalized.push(LeanCoverageFormalizedEntry {
                    argument: argument.name.clone(),
                    mode: "lean_atom_projection_mirror_entailment".to_string(),
                    specification_assertions: specification_values.len(),
                    world_assertions: world_values.len(),
                    requirement_assertions: requirement_values.len(),
                });
            }
            FormalCoverageDecision::Skipped { reason } => {
                skipped.push(LeanCoverageSkippedEntry {
                    argument: argument.name.clone(),
                    reason: reason.to_string(),
                });
            }
        }
    }

    serde_json::to_string_pretty(&LeanCoverageReport {
        problem: problem.name.clone(),
        total_correctness_arguments: problem.correctness_arguments.len(),
        formalized_count: formalized.len(),
        skipped_count: skipped.len(),
        formalized,
        skipped,
    })
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

    output.push_str("def machineDomainCount : Nat :=\n");
    output.push_str("  (domains.filter fun d => d.role = DomainRole.machine).length\n\n");
    output.push_str("def interfaceControllersDeclaredBool : Bool :=\n");
    output.push_str("  interfaces.all fun iface =>\n");
    output.push_str("    iface.phenomena.all fun p => p.controlledBy ∈ iface.connects\n\n");
    output
        .push_str("def assertionSetHasScope (name : String) (scope : AssertionScope) : Bool :=\n");
    output.push_str("  assertionSets.any fun set => set.name = name && set.scope = scope\n\n");
    output.push_str("def correctnessArgumentsStructuredBool : Bool :=\n");
    output.push_str("  correctnessArguments.all fun arg =>\n");
    output.push_str(
        "    assertionSetHasScope arg.specificationSet AssertionScope.specification &&\n",
    );
    output.push_str("    assertionSetHasScope arg.worldSet AssertionScope.worldProperties &&\n");
    output.push_str(
        "    assertionSetHasScope arg.requirementSet AssertionScope.requirementAssertions\n\n",
    );
    output.push_str("/-- Model contains exactly one machine domain. -/\n");
    output.push_str("theorem machineBoundaryCaptured : machineDomainCount = 1 := by\n");
    output.push_str("  decide\n\n");
    output.push_str("/-- Every interface phenomenon is controlled by a connected domain. -/\n");
    output.push_str(
        "theorem interfaceControllersDeclared : interfaceControllersDeclaredBool = true := by\n",
    );
    output.push_str("  decide\n\n");
    output.push_str(
        "/-- Every correctness argument references assertion sets with valid W/S/R scopes. -/\n",
    );
    output.push_str("theorem correctnessArgumentsStructured : correctnessArgumentsStructuredBool = true := by\n");
    output.push_str("  decide\n\n");
    emit_formal_correctness_argument_proofs(problem, &mut output);
    output.push_str("end ProblemFramesGenerated\n");

    output
}

#[cfg(test)]
mod tests {
    use super::{generate_lean_coverage_json, generate_lean_model};
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

        let coverage_json = generate_lean_coverage_json(&problem).expect("coverage json");
        assert!(coverage_json.contains("\"formalized_count\": 0"));
        assert!(coverage_json.contains("\"reason\": \"missing_world_set\""));
    }

    #[test]
    fn formalizes_mixed_assertion_sets_via_leanatom_projection() {
        let problem = Problem {
            name: "Projection".to_string(),
            span: span(),
            imports: vec![],
            domains: vec![Domain {
                name: "Machine".to_string(),
                kind: DomainKind::Causal,
                role: DomainRole::Machine,
                marks: vec![],
                span: span(),
                source_path: None,
            }],
            interfaces: vec![],
            requirements: vec![],
            subproblems: vec![],
            assertion_sets: vec![
                AssertionSet {
                    name: "W".to_string(),
                    scope: AssertionScope::WorldProperties,
                    assertions: vec![
                        Assertion {
                            text: "world narrative".to_string(),
                            language: Some("LTL".to_string()),
                            span: span(),
                        },
                        Assertion {
                            text: "WorldFact".to_string(),
                            language: Some("LeanAtom".to_string()),
                            span: span(),
                        },
                    ],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "S".to_string(),
                    scope: AssertionScope::Specification,
                    assertions: vec![
                        Assertion {
                            text: "spec narrative".to_string(),
                            language: Some("LTL".to_string()),
                            span: span(),
                        },
                        Assertion {
                            text: "SpecFact".to_string(),
                            language: Some("LeanAtom".to_string()),
                            span: span(),
                        },
                    ],
                    span: span(),
                    source_path: None,
                },
                AssertionSet {
                    name: "R".to_string(),
                    scope: AssertionScope::RequirementAssertions,
                    assertions: vec![
                        Assertion {
                            text: "requirement narrative".to_string(),
                            language: Some("LTL".to_string()),
                            span: span(),
                        },
                        Assertion {
                            text: "SpecFact".to_string(),
                            language: Some("LeanAtom".to_string()),
                            span: span(),
                        },
                    ],
                    span: span(),
                    source_path: None,
                },
            ],
            correctness_arguments: vec![CorrectnessArgument {
                name: "A_projection".to_string(),
                specification_set: "S".to_string(),
                world_set: "W".to_string(),
                requirement_set: "R".to_string(),
                specification_ref: reference("S"),
                world_ref: reference("W"),
                requirement_ref: reference("R"),
                span: span(),
                source_path: None,
            }],
        };

        let lean_model = generate_lean_model(&problem);
        assert!(lean_model.contains("theorem A_projectionEntailment"));

        let coverage_json = generate_lean_coverage_json(&problem).expect("coverage json");
        assert!(coverage_json.contains("\"formalized_count\": 1"));
        assert!(coverage_json.contains("\"mode\": \"lean_atom_projection_mirror_entailment\""));
    }
}
