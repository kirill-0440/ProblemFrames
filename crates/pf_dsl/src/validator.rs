use crate::ast::*;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

const FORMAL_ARGUMENT_MARK: &str = "formal.argument";
const MDA_LAYER_MARK: &str = "mda.layer";

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Domain '{0}' referenced in interface '{1}' but not defined.")]
    UndefinedDomainInInterface(String, String, Span, usize),
    #[error("Domain '{0}' referenced in requirement '{1}' but not defined.")]
    UndefinedDomainInRequirement(String, String, Span),
    #[error("Requirement '{0}' with frame '{1}': {2}")]
    InvalidFrameDomain(String, String, String, Span),
    #[error("Duplicate domain definition: '{0}'")]
    DuplicateDomain(String, Span, usize),
    #[error("Duplicate interface definition: '{0}'")]
    DuplicateInterface(String, Span, usize),
    #[error("Duplicate requirement definition: '{0}'")]
    DuplicateRequirement(String, Span, usize),
    #[error("Missing connection between '{0}' and '{1}' required by frame '{2}'")]
    MissingConnection(String, String, String, Span, usize),
    #[error("Invalid causality: Phenomenon '{0}' ({1:?}) cannot originate from '{2}' ({3}).")]
    InvalidCausality(String, PhenomenonType, String, String, Span, usize),
    #[error("Requirement '{0}' is missing required field '{1}'.")]
    MissingRequiredField(String, String, Span),
    #[error("Requirement '{0}' uses unsupported frame '{1}'.")]
    UnsupportedFrame(String, String, Span),
    #[error("Domain '{0}' has invalid role/kind combination: {1}")]
    InvalidDomainRole(String, String, Span),
    #[error("Interface '{0}' must connect at least two domains.")]
    InterfaceInsufficientConnections(String, Span, usize),
    #[error("Interface '{0}' must declare at least one phenomenon.")]
    InterfaceWithoutPhenomena(String, Span, usize),
    #[error("Phenomenon '{0}' in interface '{1}' uses controller '{2}' that is not in interface connects list.")]
    InterfaceControllerMismatch(String, String, String, Span, usize),
    #[error("Requirement '{0}' cannot reference machine domain '{1}' in strict PF mode.")]
    RequirementReferencesMachine(String, String, Span),
    #[error("Subproblem '{0}' is missing required field '{1}'.")]
    MissingSubproblemField(String, String, Span),
    #[error("Domain '{0}' referenced in subproblem '{1}' but not defined.")]
    UndefinedDomainInSubproblem(String, String, Span),
    #[error("Requirement '{0}' referenced in subproblem '{1}' but not defined.")]
    UndefinedRequirementInSubproblem(String, String, Span),
    #[error("Duplicate subproblem definition: '{0}'")]
    DuplicateSubproblem(String, Span, usize),
    #[error("Subproblem '{0}' is invalid: {1}")]
    InvalidSubproblem(String, String, Span),
    #[error("Duplicate assertion set definition: '{0}'")]
    DuplicateAssertionSet(String, Span, usize),
    #[error("Assertion set '{0}' must contain at least one assertion.")]
    EmptyAssertionSet(String, Span),
    #[error("Correctness argument '{0}' is invalid: {1}")]
    InvalidCorrectnessArgument(String, String, Span),
    #[error("Duplicate correctness argument definition: '{0}'")]
    DuplicateCorrectnessArgument(String, Span, usize),
    #[error(
        "Specification assertion set '{0}' references non-shared interface vocabulary '{1}'. Use [[Interface.Phenomenon]] from declared shared phenomena."
    )]
    InvalidSpecificationVocabulary(String, String, Span),
    #[error("Domain '{0}' has invalid mark contract: {1}")]
    InvalidDomainMark(String, String, Span),
    #[error("Requirement '{0}' has invalid mark contract: {1}")]
    InvalidRequirementMark(String, String, Span),
}

#[derive(Debug)]
pub struct ValidationIssue {
    pub error: ValidationError,
    pub source_path: Option<PathBuf>,
}

fn is_connected(problem: &Problem, domain1: &str, domain2: &str) -> bool {
    problem.interfaces.iter().any(|i| {
        i.shared_phenomena.iter().any(|p| {
            (p.from.name == domain1 && p.to.name == domain2)
                || (p.from.name == domain2 && p.to.name == domain1)
        })
    })
}

fn is_machine(domain: &Domain) -> bool {
    domain.role == DomainRole::Machine
}

fn find_domain<'a>(problem: &'a Problem, name: &str) -> Option<&'a Domain> {
    problem.domains.iter().find(|domain| domain.name == name)
}

fn connected_to_machine(problem: &Problem, domain_name: &str) -> bool {
    problem.domains.iter().any(|domain| {
        domain.role == DomainRole::Machine && is_connected(problem, domain_name, &domain.name)
    })
}

fn shared_interface_vocabulary(problem: &Problem) -> HashSet<String> {
    let mut vocabulary = HashSet::new();
    for interface in &problem.interfaces {
        for phenomenon in &interface.shared_phenomena {
            vocabulary.insert(format!("{}.{}", interface.name, phenomenon.name));
        }
    }
    vocabulary
}

fn extract_interface_vocab_tokens(assertion: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while let Some(start_offset) = assertion[cursor..].find("[[") {
        let start = cursor + start_offset + 2;
        let Some(end_offset) = assertion[start..].find("]]") else {
            break;
        };
        let end = start + end_offset;
        let token = assertion[start..end].trim();
        if !token.is_empty() {
            tokens.push(token.to_string());
        }
        cursor = end + 2;
    }

    tokens
}

fn validate_domain_marks(domain: &Domain, errors: &mut Vec<ValidationError>) {
    if domain.marks.is_empty() {
        return;
    }

    let allowed_marks = [
        "ddd.bounded_context",
        "ddd.aggregate_root",
        "ddd.value_object",
        "ddd.external_system",
        "sysml.block",
        "sysml.port",
        "sysml.signal",
    ];

    let mut seen_marks = HashSet::new();
    let mut has_bounded_context = false;

    for mark in &domain.marks {
        if !allowed_marks.contains(&mark.name.as_str()) {
            errors.push(ValidationError::InvalidDomainMark(
                domain.name.clone(),
                format!("unsupported mark '{}'", mark.name),
                mark.span,
            ));
            continue;
        }

        if !seen_marks.insert(mark.name.as_str()) {
            errors.push(ValidationError::InvalidDomainMark(
                domain.name.clone(),
                format!("duplicate mark '{}'", mark.name),
                mark.span,
            ));
            continue;
        }

        match mark.name.as_str() {
            "ddd.bounded_context" => {
                has_bounded_context = true;
                let is_missing = mark
                    .value
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true);
                if is_missing {
                    errors.push(ValidationError::InvalidDomainMark(
                        domain.name.clone(),
                        "mark 'ddd.bounded_context' requires non-empty string value".to_string(),
                        mark.span,
                    ));
                }
            }
            "ddd.aggregate_root"
            | "ddd.value_object"
            | "ddd.external_system"
            | "sysml.block"
            | "sysml.port"
            | "sysml.signal" => {
                if mark.value.is_some() {
                    errors.push(ValidationError::InvalidDomainMark(
                        domain.name.clone(),
                        format!("mark '{}' does not accept a value", mark.name),
                        mark.span,
                    ));
                }
            }
            _ => {}
        }
    }

    let has_aggregate_root = seen_marks.contains("ddd.aggregate_root");
    let has_value_object = seen_marks.contains("ddd.value_object");
    if has_aggregate_root && has_value_object {
        errors.push(ValidationError::InvalidDomainMark(
            domain.name.clone(),
            "marks 'ddd.aggregate_root' and 'ddd.value_object' are mutually exclusive".to_string(),
            domain.span,
        ));
    }

    if (has_aggregate_root || has_value_object) && !has_bounded_context {
        errors.push(ValidationError::InvalidDomainMark(
            domain.name.clone(),
            "marks 'ddd.aggregate_root'/'ddd.value_object' require 'ddd.bounded_context'"
                .to_string(),
            domain.span,
        ));
    }
}

fn validate_requirement_marks(requirement: &Requirement, errors: &mut Vec<ValidationError>) {
    if requirement.marks.is_empty() {
        return;
    }

    let allowed_marks = [
        "sysml.requirement",
        "ddd.application_service",
        FORMAL_ARGUMENT_MARK,
        MDA_LAYER_MARK,
    ];
    let mut seen_marks = HashSet::new();

    for mark in &requirement.marks {
        if !allowed_marks.contains(&mark.name.as_str()) {
            errors.push(ValidationError::InvalidRequirementMark(
                requirement.name.clone(),
                format!("unsupported mark '{}'", mark.name),
                mark.span,
            ));
            continue;
        }

        if !seen_marks.insert(mark.name.as_str()) {
            errors.push(ValidationError::InvalidRequirementMark(
                requirement.name.clone(),
                format!("duplicate mark '{}'", mark.name),
                mark.span,
            ));
            continue;
        }

        match mark.name.as_str() {
            "sysml.requirement" => {
                if mark.value.is_some() {
                    errors.push(ValidationError::InvalidRequirementMark(
                        requirement.name.clone(),
                        "mark 'sysml.requirement' does not accept a value".to_string(),
                        mark.span,
                    ));
                }
            }
            "ddd.application_service" => {
                let is_missing = mark
                    .value
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true);
                if is_missing {
                    errors.push(ValidationError::InvalidRequirementMark(
                        requirement.name.clone(),
                        "mark 'ddd.application_service' requires non-empty string value"
                            .to_string(),
                        mark.span,
                    ));
                }
            }
            FORMAL_ARGUMENT_MARK => {
                let is_missing = mark
                    .value
                    .as_ref()
                    .map(|value| value.trim().is_empty())
                    .unwrap_or(true);
                if is_missing {
                    errors.push(ValidationError::InvalidRequirementMark(
                        requirement.name.clone(),
                        format!(
                            "mark '{}' requires non-empty string value",
                            FORMAL_ARGUMENT_MARK
                        ),
                        mark.span,
                    ));
                }
            }
            MDA_LAYER_MARK => {
                let value = mark.value.as_ref().map(|value| value.trim()).unwrap_or("");
                if value.is_empty() {
                    errors.push(ValidationError::InvalidRequirementMark(
                        requirement.name.clone(),
                        format!("mark '{}' requires non-empty string value", MDA_LAYER_MARK),
                        mark.span,
                    ));
                } else if value != "CIM" && value != "PIM" && value != "PSM" {
                    errors.push(ValidationError::InvalidRequirementMark(
                        requirement.name.clone(),
                        format!("mark '{}' must be one of: CIM, PIM, PSM", MDA_LAYER_MARK),
                        mark.span,
                    ));
                }
            }
            _ => {}
        }
    }
}

fn requirement_formal_argument_mark(requirement: &Requirement) -> Option<(String, Span)> {
    requirement.marks.iter().find_map(|mark| {
        if mark.name != FORMAL_ARGUMENT_MARK {
            return None;
        }

        mark.value
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| (value.to_string(), mark.span))
    })
}

pub fn validate(problem: &Problem) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    let mut defined_domains = HashSet::new();
    let mut machine_count = 0_usize;

    for (index, domain) in problem.domains.iter().enumerate() {
        if defined_domains.contains(&domain.name) {
            errors.push(ValidationError::DuplicateDomain(
                domain.name.clone(),
                domain.span,
                index,
            ));
        } else {
            defined_domains.insert(domain.name.clone());
        }

        if domain.role == DomainRole::Machine {
            machine_count += 1;
            if domain.kind == DomainKind::Lexical {
                errors.push(ValidationError::InvalidDomainRole(
                    domain.name.clone(),
                    "lexical domains cannot have machine role".to_string(),
                    domain.span,
                ));
            }
        }

        validate_domain_marks(domain, &mut errors);
    }

    if machine_count > 1 {
        errors.push(ValidationError::InvalidDomainRole(
            "<problem>".to_string(),
            format!("expected at most one machine domain, found {machine_count}"),
            problem.span,
        ));
    } else if machine_count == 0 && !problem.requirements.is_empty() {
        errors.push(ValidationError::InvalidDomainRole(
            "<problem>".to_string(),
            "expected one machine domain when requirements are present".to_string(),
            problem.span,
        ));
    }

    let mut defined_interfaces = HashSet::new();
    for (index, interface) in problem.interfaces.iter().enumerate() {
        if defined_interfaces.contains(&interface.name) {
            errors.push(ValidationError::DuplicateInterface(
                interface.name.clone(),
                interface.span,
                index,
            ));
        } else {
            defined_interfaces.insert(interface.name.clone());
        }
    }

    for (interface_index, interface) in problem.interfaces.iter().enumerate() {
        if interface.connects.len() < 2 {
            errors.push(ValidationError::InterfaceInsufficientConnections(
                interface.name.clone(),
                interface.span,
                interface_index,
            ));
        }
        if interface.shared_phenomena.is_empty() {
            errors.push(ValidationError::InterfaceWithoutPhenomena(
                interface.name.clone(),
                interface.span,
                interface_index,
            ));
        }

        for connected in &interface.connects {
            if !defined_domains.contains(&connected.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    connected.name.clone(),
                    interface.name.clone(),
                    connected.span,
                    interface_index,
                ));
            }
        }

        for phenomenon in &interface.shared_phenomena {
            if !defined_domains.contains(&phenomenon.from.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.from.name.clone(),
                    interface.name.clone(),
                    phenomenon.from.span,
                    interface_index,
                ));
            }
            if !defined_domains.contains(&phenomenon.to.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.to.name.clone(),
                    interface.name.clone(),
                    phenomenon.to.span,
                    interface_index,
                ));
            }
            if !defined_domains.contains(&phenomenon.controlled_by.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.controlled_by.name.clone(),
                    interface.name.clone(),
                    phenomenon.controlled_by.span,
                    interface_index,
                ));
            }

            let connected_names: HashSet<&str> = interface
                .connects
                .iter()
                .map(|reference| reference.name.as_str())
                .collect();
            if !connected_names.contains(phenomenon.controlled_by.name.as_str()) {
                errors.push(ValidationError::InterfaceControllerMismatch(
                    phenomenon.name.clone(),
                    interface.name.clone(),
                    phenomenon.controlled_by.name.clone(),
                    phenomenon.controlled_by.span,
                    interface_index,
                ));
            }
            if phenomenon.controlled_by.name != phenomenon.from.name {
                errors.push(ValidationError::InterfaceControllerMismatch(
                    phenomenon.name.clone(),
                    interface.name.clone(),
                    phenomenon.controlled_by.name.clone(),
                    phenomenon.controlled_by.span,
                    interface_index,
                ));
            }

            if let Some(from_domain) = problem
                .domains
                .iter()
                .find(|d| d.name == phenomenon.from.name)
            {
                match phenomenon.type_ {
                    PhenomenonType::Event | PhenomenonType::Command => {
                        if from_domain.kind == DomainKind::Lexical
                            || from_domain.role == DomainRole::Designed
                        {
                            errors.push(ValidationError::InvalidCausality(
                                phenomenon.name.clone(),
                                phenomenon.type_.clone(),
                                from_domain.name.clone(),
                                format!("{:?}/{:?}", from_domain.kind, from_domain.role),
                                phenomenon.span,
                                interface_index,
                            ));
                        }
                        if phenomenon.type_ == PhenomenonType::Command
                            && from_domain.kind != DomainKind::Biddable
                        {
                            errors.push(ValidationError::InvalidCausality(
                                phenomenon.name.clone(),
                                phenomenon.type_.clone(),
                                from_domain.name.clone(),
                                format!("{:?}/{:?}", from_domain.kind, from_domain.role),
                                phenomenon.span,
                                interface_index,
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut requirement_names = HashSet::new();
    for (index, req) in problem.requirements.iter().enumerate() {
        if !requirement_names.insert(req.name.clone()) {
            errors.push(ValidationError::DuplicateRequirement(
                req.name.clone(),
                req.span,
                index,
            ));
        }
    }

    for req in &problem.requirements {
        validate_requirement_marks(req, &mut errors);

        if let Some(ref c) = req.constrains {
            if !defined_domains.contains(&c.name) {
                errors.push(ValidationError::UndefinedDomainInRequirement(
                    c.name.clone(),
                    req.name.clone(),
                    c.span,
                ));
            }
        }

        if let Some(ref r) = req.reference {
            if !defined_domains.contains(&r.name) {
                errors.push(ValidationError::UndefinedDomainInRequirement(
                    r.name.clone(),
                    req.name.clone(),
                    r.span,
                ));
            }
        }
    }

    let mut subproblem_names = HashSet::new();
    for (index, subproblem) in problem.subproblems.iter().enumerate() {
        if !subproblem_names.insert(subproblem.name.clone()) {
            errors.push(ValidationError::DuplicateSubproblem(
                subproblem.name.clone(),
                subproblem.span,
                index,
            ));
        }
    }

    for subproblem in &problem.subproblems {
        if subproblem.machine.is_none() {
            errors.push(ValidationError::MissingSubproblemField(
                subproblem.name.clone(),
                "machine".to_string(),
                subproblem.span,
            ));
        }
        if subproblem.participants.is_empty() {
            errors.push(ValidationError::MissingSubproblemField(
                subproblem.name.clone(),
                "participants".to_string(),
                subproblem.span,
            ));
        }
        if subproblem.requirements.is_empty() {
            errors.push(ValidationError::MissingSubproblemField(
                subproblem.name.clone(),
                "requirements".to_string(),
                subproblem.span,
            ));
        }

        if let Some(machine) = &subproblem.machine {
            if !defined_domains.contains(&machine.name) {
                errors.push(ValidationError::UndefinedDomainInSubproblem(
                    machine.name.clone(),
                    subproblem.name.clone(),
                    machine.span,
                ));
            } else if let Some(domain) = find_domain(problem, &machine.name) {
                if domain.role != DomainRole::Machine {
                    errors.push(ValidationError::InvalidSubproblem(
                        subproblem.name.clone(),
                        format!("machine '{}' must have role machine", machine.name),
                        machine.span,
                    ));
                }
            }
        }

        let mut participant_names = HashSet::new();
        for participant in &subproblem.participants {
            if !defined_domains.contains(&participant.name) {
                errors.push(ValidationError::UndefinedDomainInSubproblem(
                    participant.name.clone(),
                    subproblem.name.clone(),
                    participant.span,
                ));
            } else {
                participant_names.insert(participant.name.as_str());
            }
        }

        if let Some(machine) = &subproblem.machine {
            if !participant_names.contains(machine.name.as_str()) {
                errors.push(ValidationError::InvalidSubproblem(
                    subproblem.name.clone(),
                    format!("participants must include machine '{}'", machine.name),
                    subproblem.span,
                ));
            }
        }

        for requirement_ref in &subproblem.requirements {
            if !problem
                .requirements
                .iter()
                .any(|requirement| requirement.name == requirement_ref.name)
            {
                errors.push(ValidationError::UndefinedRequirementInSubproblem(
                    requirement_ref.name.clone(),
                    subproblem.name.clone(),
                    requirement_ref.span,
                ));
            }
        }

        for requirement_ref in &subproblem.requirements {
            if let Some(requirement) = problem
                .requirements
                .iter()
                .find(|requirement| requirement.name == requirement_ref.name)
            {
                if let Some(constrains) = &requirement.constrains {
                    if !participant_names.contains(constrains.name.as_str()) {
                        errors.push(ValidationError::InvalidSubproblem(
                            subproblem.name.clone(),
                            format!(
                                "requirement '{}' constrains '{}' outside participants",
                                requirement.name, constrains.name
                            ),
                            requirement_ref.span,
                        ));
                    }
                }
                if let Some(reference) = &requirement.reference {
                    if !participant_names.contains(reference.name.as_str()) {
                        errors.push(ValidationError::InvalidSubproblem(
                            subproblem.name.clone(),
                            format!(
                                "requirement '{}' references '{}' outside participants",
                                requirement.name, reference.name
                            ),
                            requirement_ref.span,
                        ));
                    }
                }
            }
        }
    }

    let mut assertion_set_names = HashSet::new();
    for (index, assertion_set) in problem.assertion_sets.iter().enumerate() {
        if !assertion_set_names.insert(assertion_set.name.clone()) {
            errors.push(ValidationError::DuplicateAssertionSet(
                assertion_set.name.clone(),
                assertion_set.span,
                index,
            ));
        }
        if assertion_set.assertions.is_empty() {
            errors.push(ValidationError::EmptyAssertionSet(
                assertion_set.name.clone(),
                assertion_set.span,
            ));
        }
    }

    let interface_vocabulary = shared_interface_vocabulary(problem);
    for assertion_set in &problem.assertion_sets {
        if assertion_set.scope != AssertionScope::Specification {
            continue;
        }

        for assertion in &assertion_set.assertions {
            for token in extract_interface_vocab_tokens(&assertion.text) {
                if !interface_vocabulary.contains(&token) {
                    errors.push(ValidationError::InvalidSpecificationVocabulary(
                        assertion_set.name.clone(),
                        token,
                        assertion.span,
                    ));
                }
            }
        }
    }

    let mut correctness_argument_names = HashSet::new();
    for (index, argument) in problem.correctness_arguments.iter().enumerate() {
        if !correctness_argument_names.insert(argument.name.clone()) {
            errors.push(ValidationError::DuplicateCorrectnessArgument(
                argument.name.clone(),
                argument.span,
                index,
            ));
        }
    }

    for argument in &problem.correctness_arguments {
        let specification_set = problem
            .assertion_sets
            .iter()
            .find(|assertion_set| assertion_set.name == argument.specification_set);
        let world_set = problem
            .assertion_sets
            .iter()
            .find(|assertion_set| assertion_set.name == argument.world_set);
        let requirement_set = problem
            .assertion_sets
            .iter()
            .find(|assertion_set| assertion_set.name == argument.requirement_set);

        match specification_set {
            Some(set) if matches!(set.scope, AssertionScope::Specification) => {}
            Some(set) => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!(
                    "specification set '{}' has wrong scope {:?}",
                    argument.specification_set, set.scope
                ),
                argument.span,
            )),
            None => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!(
                    "specification set '{}' is not defined",
                    argument.specification_set
                ),
                argument.span,
            )),
        }

        match world_set {
            Some(set) if matches!(set.scope, AssertionScope::WorldProperties) => {}
            Some(set) => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!(
                    "world set '{}' has wrong scope {:?}",
                    argument.world_set, set.scope
                ),
                argument.span,
            )),
            None => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!("world set '{}' is not defined", argument.world_set),
                argument.span,
            )),
        }

        match requirement_set {
            Some(set) if matches!(set.scope, AssertionScope::RequirementAssertions) => {}
            Some(set) => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!(
                    "requirement set '{}' has wrong scope {:?}",
                    argument.requirement_set, set.scope
                ),
                argument.span,
            )),
            None => errors.push(ValidationError::InvalidCorrectnessArgument(
                argument.name.clone(),
                format!(
                    "requirement set '{}' is not defined",
                    argument.requirement_set
                ),
                argument.span,
            )),
        }
    }

    for requirement in &problem.requirements {
        if let Some((argument_name, span)) = requirement_formal_argument_mark(requirement) {
            if !correctness_argument_names.contains(argument_name.as_str()) {
                errors.push(ValidationError::InvalidRequirementMark(
                    requirement.name.clone(),
                    format!(
                        "mark '{}' references undefined correctness argument '{}'",
                        FORMAL_ARGUMENT_MARK, argument_name
                    ),
                    span,
                ));
            }
        }
    }

    for req in &problem.requirements {
        if let Some(ref r) = req.reference {
            if let Some(domain) = problem.domains.iter().find(|d| d.name == r.name) {
                if is_machine(domain) {
                    errors.push(ValidationError::RequirementReferencesMachine(
                        req.name.clone(),
                        domain.name.clone(),
                        r.span,
                    ));
                }
            }
        }

        if let Some(ref c) = req.constrains {
            if let Some(domain) = problem.domains.iter().find(|d| d.name == c.name) {
                if is_machine(domain) {
                    errors.push(ValidationError::RequirementReferencesMachine(
                        req.name.clone(),
                        domain.name.clone(),
                        c.span,
                    ));
                }
                if domain.kind == DomainKind::Biddable {
                    errors.push(ValidationError::InvalidFrameDomain(
                        req.name.clone(),
                        format!("{:?}", req.frame),
                        format!("constrained domain '{}' cannot be biddable", domain.name),
                        c.span,
                    ));
                }
            }
        }
    }

    for (req_index, req) in problem.requirements.iter().enumerate() {
        match &req.frame {
            FrameType::Custom(frame_name) if frame_name.is_empty() => {
                errors.push(ValidationError::MissingRequiredField(
                    req.name.clone(),
                    "frame".to_string(),
                    req.span,
                ));
                continue;
            }
            FrameType::Custom(frame_name) => {
                errors.push(ValidationError::UnsupportedFrame(
                    req.name.clone(),
                    frame_name.clone(),
                    req.span,
                ));
                continue;
            }
            _ => {}
        }

        match req.frame {
            FrameType::CommandedBehavior => {
                if req.reference.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "reference".to_string(),
                        req.span,
                    ));
                }
                if req.constrains.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "constrains".to_string(),
                        req.span,
                    ));
                }

                if let Some(ref r) = req.reference {
                    if let Some(domain) = find_domain(problem, &r.name) {
                        if domain.kind != DomainKind::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "CommandedBehavior".to_string(),
                                format!(
                                    "reference domain '{}' should be biddable, found {:?}/{:?}",
                                    r.name, domain.kind, domain.role
                                ),
                                r.span,
                            ));
                        }

                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "CommandedBehavior".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }
            }
            FrameType::RequiredBehavior => {
                if req.constrains.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "constrains".to_string(),
                        req.span,
                    ));
                }

                if let Some(ref c) = req.constrains {
                    if let Some(domain) = find_domain(problem, &c.name) {
                        if domain.kind != DomainKind::Causal {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "RequiredBehavior".to_string(),
                                format!(
                                    "constrained domain '{}' should be causal, found {:?}/{:?}",
                                    c.name, domain.kind, domain.role
                                ),
                                c.span,
                            ));
                        }

                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "RequiredBehavior".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }
            }
            FrameType::InformationDisplay => {
                if req.reference.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "reference".to_string(),
                        req.span,
                    ));
                }
                if req.constrains.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "constrains".to_string(),
                        req.span,
                    ));
                }

                if let Some(ref r) = req.reference {
                    if let Some(domain) = find_domain(problem, &r.name) {
                        if domain.kind != DomainKind::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "InformationDisplay".to_string(),
                                format!(
                                    "reference domain '{}' should be biddable, found {:?}/{:?}",
                                    r.name, domain.kind, domain.role
                                ),
                                r.span,
                            ));
                        }
                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "InformationDisplay".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }

                if let Some(ref c) = req.constrains {
                    if let Some(domain) = find_domain(problem, &c.name) {
                        if domain.kind == DomainKind::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "InformationDisplay".to_string(),
                                format!("constrained domain '{}' cannot be biddable", c.name),
                                c.span,
                            ));
                        }
                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "InformationDisplay".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }
            }
            FrameType::SimpleWorkpieces => {
                if req.reference.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "reference".to_string(),
                        req.span,
                    ));
                }
                if req.constrains.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "constrains".to_string(),
                        req.span,
                    ));
                }

                if let Some(ref r) = req.reference {
                    if let Some(domain) = find_domain(problem, &r.name) {
                        if domain.kind != DomainKind::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "SimpleWorkpieces".to_string(),
                                format!(
                                    "reference domain '{}' should be biddable, found {:?}/{:?}",
                                    r.name, domain.kind, domain.role
                                ),
                                r.span,
                            ));
                        }
                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "SimpleWorkpieces".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }

                if let Some(ref c) = req.constrains {
                    if let Some(domain) = find_domain(problem, &c.name) {
                        if domain.kind != DomainKind::Lexical {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "SimpleWorkpieces".to_string(),
                                format!(
                                    "constrained domain '{}' should be lexical, found {:?}/{:?}",
                                    c.name, domain.kind, domain.role
                                ),
                                c.span,
                            ));
                        }
                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "SimpleWorkpieces".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }
            }
            FrameType::Transformation => {
                if req.constrains.is_none() {
                    errors.push(ValidationError::MissingRequiredField(
                        req.name.clone(),
                        "constrains".to_string(),
                        req.span,
                    ));
                }

                if let Some(ref c) = req.constrains {
                    if let Some(domain) = find_domain(problem, &c.name) {
                        if domain.kind != DomainKind::Lexical {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "Transformation".to_string(),
                                format!(
                                    "constrained domain '{}' should be lexical, found {:?}/{:?}",
                                    c.name, domain.kind, domain.role
                                ),
                                c.span,
                            ));
                        }
                        if !connected_to_machine(problem, &domain.name) {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "Transformation".to_string(),
                                req.span,
                                req_index,
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn validation_error_span(error: &ValidationError) -> Span {
    match error {
        ValidationError::UndefinedDomainInInterface(_, _, span, _)
        | ValidationError::UndefinedDomainInRequirement(_, _, span)
        | ValidationError::InvalidFrameDomain(_, _, _, span)
        | ValidationError::DuplicateDomain(_, span, _)
        | ValidationError::DuplicateInterface(_, span, _)
        | ValidationError::DuplicateRequirement(_, span, _)
        | ValidationError::MissingConnection(_, _, _, span, _)
        | ValidationError::InvalidCausality(_, _, _, _, span, _)
        | ValidationError::MissingRequiredField(_, _, span)
        | ValidationError::UnsupportedFrame(_, _, span)
        | ValidationError::InvalidDomainRole(_, _, span)
        | ValidationError::InterfaceInsufficientConnections(_, span, _)
        | ValidationError::InterfaceWithoutPhenomena(_, span, _)
        | ValidationError::InterfaceControllerMismatch(_, _, _, span, _)
        | ValidationError::RequirementReferencesMachine(_, _, span)
        | ValidationError::MissingSubproblemField(_, _, span)
        | ValidationError::UndefinedDomainInSubproblem(_, _, span)
        | ValidationError::UndefinedRequirementInSubproblem(_, _, span)
        | ValidationError::DuplicateSubproblem(_, span, _)
        | ValidationError::InvalidSubproblem(_, _, span)
        | ValidationError::DuplicateAssertionSet(_, span, _)
        | ValidationError::EmptyAssertionSet(_, span)
        | ValidationError::InvalidCorrectnessArgument(_, _, span)
        | ValidationError::InvalidSpecificationVocabulary(_, _, span)
        | ValidationError::InvalidDomainMark(_, _, span)
        | ValidationError::InvalidRequirementMark(_, _, span) => *span,
        ValidationError::DuplicateCorrectnessArgument(_, span, _) => *span,
    }
}

fn source_path_for_error(problem: &Problem, error: &ValidationError) -> Option<PathBuf> {
    let requirement_matches_span = |requirement: &Requirement, span: Span| {
        requirement.span == span
            || requirement
                .constrains
                .as_ref()
                .map(|reference| reference.span == span)
                .unwrap_or(false)
            || requirement
                .reference
                .as_ref()
                .map(|reference| reference.span == span)
                .unwrap_or(false)
    };

    let subproblem_matches_span = |subproblem: &Subproblem, span: Span| {
        subproblem.span == span
            || subproblem
                .machine
                .as_ref()
                .map(|reference| reference.span == span)
                .unwrap_or(false)
            || subproblem
                .participants
                .iter()
                .any(|reference| reference.span == span)
            || subproblem
                .requirements
                .iter()
                .any(|reference| reference.span == span)
    };

    match error {
        ValidationError::UndefinedDomainInInterface(_, _, _, index)
        | ValidationError::InterfaceInsufficientConnections(_, _, index)
        | ValidationError::InterfaceWithoutPhenomena(_, _, index)
        | ValidationError::InterfaceControllerMismatch(_, _, _, _, index)
        | ValidationError::InvalidCausality(_, _, _, _, _, index) => problem
            .interfaces
            .get(*index)
            .and_then(|interface| interface.source_path.clone()),
        ValidationError::UndefinedDomainInRequirement(domain_name, requirement_name, span) => {
            problem
                .requirements
                .iter()
                .find(|requirement| {
                    requirement.name == *requirement_name
                        && ((requirement
                            .constrains
                            .as_ref()
                            .map(|reference| {
                                reference.name == *domain_name && reference.span == *span
                            })
                            .unwrap_or(false))
                            || (requirement
                                .reference
                                .as_ref()
                                .map(|reference| {
                                    reference.name == *domain_name && reference.span == *span
                                })
                                .unwrap_or(false))
                            || requirement.span == *span)
                })
                .or_else(|| {
                    problem
                        .requirements
                        .iter()
                        .find(|requirement| requirement.name == *requirement_name)
                })
                .and_then(|requirement| requirement.source_path.clone())
        }
        ValidationError::MissingRequiredField(requirement_name, _, span)
        | ValidationError::UnsupportedFrame(requirement_name, _, span) => problem
            .requirements
            .iter()
            .find(|requirement| requirement.name == *requirement_name && requirement.span == *span)
            .or_else(|| {
                problem
                    .requirements
                    .iter()
                    .find(|requirement| requirement.name == *requirement_name)
            })
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::RequirementReferencesMachine(requirement_name, domain_name, span) => {
            problem
                .requirements
                .iter()
                .find(|requirement| {
                    requirement.name == *requirement_name
                        && ((requirement
                            .constrains
                            .as_ref()
                            .map(|reference| {
                                reference.name == *domain_name && reference.span == *span
                            })
                            .unwrap_or(false))
                            || (requirement
                                .reference
                                .as_ref()
                                .map(|reference| {
                                    reference.name == *domain_name && reference.span == *span
                                })
                                .unwrap_or(false))
                            || requirement_matches_span(requirement, *span))
                })
                .or_else(|| {
                    problem
                        .requirements
                        .iter()
                        .find(|requirement| requirement.name == *requirement_name)
                })
                .and_then(|requirement| requirement.source_path.clone())
        }
        ValidationError::InvalidFrameDomain(requirement_name, _, _, span) => problem
            .requirements
            .iter()
            .find(|requirement| {
                requirement.name == *requirement_name
                    && requirement_matches_span(requirement, *span)
            })
            .or_else(|| {
                problem
                    .requirements
                    .iter()
                    .find(|requirement| requirement.name == *requirement_name)
            })
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::MissingConnection(_, _, _, _, index) => problem
            .requirements
            .get(*index)
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::DuplicateDomain(_, _, index) => problem
            .domains
            .get(*index)
            .and_then(|domain| domain.source_path.clone()),
        ValidationError::DuplicateInterface(_, _, index) => problem
            .interfaces
            .get(*index)
            .and_then(|interface| interface.source_path.clone()),
        ValidationError::DuplicateRequirement(_, _, index) => problem
            .requirements
            .get(*index)
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::InvalidDomainRole(domain_name, _, span) => {
            if domain_name == "<problem>" {
                return None;
            }
            problem
                .domains
                .iter()
                .find(|domain| domain.name == *domain_name && domain.span == *span)
                .and_then(|domain| domain.source_path.clone())
        }
        ValidationError::DuplicateAssertionSet(_, _, index) => problem
            .assertion_sets
            .get(*index)
            .and_then(|set| set.source_path.clone()),
        ValidationError::EmptyAssertionSet(name, span) => problem
            .assertion_sets
            .iter()
            .find(|set| set.name == *name && set.span == *span)
            .and_then(|set| set.source_path.clone()),
        ValidationError::InvalidCorrectnessArgument(name, _, span) => problem
            .correctness_arguments
            .iter()
            .find(|argument| argument.name == *name && argument.span == *span)
            .or_else(|| {
                problem
                    .correctness_arguments
                    .iter()
                    .find(|argument| argument.name == *name)
            })
            .and_then(|argument| argument.source_path.clone()),
        ValidationError::InvalidSpecificationVocabulary(name, _, span) => problem
            .assertion_sets
            .iter()
            .find(|set| {
                set.name == *name
                    && (set.span == *span || set.assertions.iter().any(|a| a.span == *span))
            })
            .or_else(|| problem.assertion_sets.iter().find(|set| set.name == *name))
            .and_then(|set| set.source_path.clone()),
        ValidationError::MissingSubproblemField(name, _, span)
        | ValidationError::InvalidSubproblem(name, _, span) => problem
            .subproblems
            .iter()
            .find(|subproblem| {
                subproblem.name == *name && subproblem_matches_span(subproblem, *span)
            })
            .or_else(|| {
                problem
                    .subproblems
                    .iter()
                    .find(|subproblem| subproblem.name == *name)
            })
            .and_then(|subproblem| subproblem.source_path.clone()),
        ValidationError::UndefinedDomainInSubproblem(domain_name, name, span) => problem
            .subproblems
            .iter()
            .find(|subproblem| {
                subproblem.name == *name
                    && ((subproblem
                        .machine
                        .as_ref()
                        .map(|reference| reference.name == *domain_name && reference.span == *span)
                        .unwrap_or(false))
                        || subproblem.participants.iter().any(|reference| {
                            reference.name == *domain_name && reference.span == *span
                        })
                        || subproblem_matches_span(subproblem, *span))
            })
            .or_else(|| {
                problem
                    .subproblems
                    .iter()
                    .find(|subproblem| subproblem.name == *name)
            })
            .and_then(|subproblem| subproblem.source_path.clone()),
        ValidationError::UndefinedRequirementInSubproblem(requirement_name, name, span) => problem
            .subproblems
            .iter()
            .find(|subproblem| {
                subproblem.name == *name
                    && (subproblem.requirements.iter().any(|reference| {
                        reference.name == *requirement_name && reference.span == *span
                    }) || subproblem_matches_span(subproblem, *span))
            })
            .or_else(|| {
                problem
                    .subproblems
                    .iter()
                    .find(|subproblem| subproblem.name == *name)
            })
            .and_then(|subproblem| subproblem.source_path.clone()),
        ValidationError::DuplicateSubproblem(_, _, index) => problem
            .subproblems
            .get(*index)
            .and_then(|subproblem| subproblem.source_path.clone()),
        ValidationError::DuplicateCorrectnessArgument(_, _, index) => problem
            .correctness_arguments
            .get(*index)
            .and_then(|argument| argument.source_path.clone()),
        ValidationError::InvalidDomainMark(name, _, span) => problem
            .domains
            .iter()
            .find(|domain| domain.name == *name && domain.span == *span)
            .or_else(|| {
                problem.domains.iter().find(|domain| {
                    domain.name == *name && domain.marks.iter().any(|mark| mark.span == *span)
                })
            })
            .or_else(|| problem.domains.iter().find(|domain| domain.name == *name))
            .and_then(|domain| domain.source_path.clone()),
        ValidationError::InvalidRequirementMark(name, _, span) => problem
            .requirements
            .iter()
            .find(|requirement| requirement.name == *name && requirement.span == *span)
            .or_else(|| {
                problem.requirements.iter().find(|requirement| {
                    requirement.name == *name
                        && requirement.marks.iter().any(|mark| mark.span == *span)
                })
            })
            .or_else(|| {
                problem
                    .requirements
                    .iter()
                    .find(|requirement| requirement.name == *name)
            })
            .and_then(|requirement| requirement.source_path.clone()),
    }
}

pub fn validate_with_sources(problem: &Problem) -> Result<(), Vec<ValidationIssue>> {
    match validate(problem) {
        Ok(()) => Ok(()),
        Err(errors) => Err(errors
            .into_iter()
            .map(|error| ValidationIssue {
                source_path: source_path_for_error(problem, &error),
                error,
            })
            .collect()),
    }
}
