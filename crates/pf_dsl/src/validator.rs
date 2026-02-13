use crate::ast::*;
use std::collections::HashSet;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Domain '{0}' referenced in interface '{1}' but not defined.")]
    UndefinedDomainInInterface(String, String, Span),
    #[error("Domain '{0}' referenced in requirement '{1}' but not defined.")]
    UndefinedDomainInRequirement(String, String, Span),
    #[error("Requirement '{0}' with frame '{1}': {2}")]
    InvalidFrameDomain(String, String, String, Span),
    #[error("Duplicate domain definition: '{0}'")]
    DuplicateDomain(String, Span),
    #[error("Duplicate interface definition: '{0}'")]
    DuplicateInterface(String, Span),
    #[error("Missing connection between '{0}' and '{1}' required by frame '{2}'")]
    MissingConnection(String, String, String, Span),
    #[error("Invalid causality: Phenomenon '{0}' ({1:?}) cannot originate from '{2}' ({3}).")]
    InvalidCausality(String, PhenomenonType, String, String, Span),
    #[error("Requirement '{0}' is missing required field '{1}'.")]
    MissingRequiredField(String, String, Span),
    #[error("Requirement '{0}' uses unsupported frame '{1}'.")]
    UnsupportedFrame(String, String, Span),
    #[error("Domain '{0}' has invalid role/kind combination: {1}")]
    InvalidDomainRole(String, String, Span),
    #[error("Interface '{0}' must connect at least two domains.")]
    InterfaceInsufficientConnections(String, Span),
    #[error("Interface '{0}' must declare at least one phenomenon.")]
    InterfaceWithoutPhenomena(String, Span),
    #[error("Phenomenon '{0}' in interface '{1}' uses controller '{2}' that is not in interface connects list.")]
    InterfaceControllerMismatch(String, String, String, Span),
    #[error("Requirement '{0}' cannot reference machine domain '{1}' in strict PF mode.")]
    RequirementReferencesMachine(String, String, Span),
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

pub fn validate(problem: &Problem) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    let mut defined_domains = HashSet::new();
    let mut machine_count = 0_usize;

    for domain in &problem.domains {
        if defined_domains.contains(&domain.name) {
            errors.push(ValidationError::DuplicateDomain(
                domain.name.clone(),
                domain.span,
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
    for interface in &problem.interfaces {
        if defined_interfaces.contains(&interface.name) {
            errors.push(ValidationError::DuplicateInterface(
                interface.name.clone(),
                interface.span,
            ));
        } else {
            defined_interfaces.insert(interface.name.clone());
        }
    }

    for interface in &problem.interfaces {
        if interface.connects.len() < 2 {
            errors.push(ValidationError::InterfaceInsufficientConnections(
                interface.name.clone(),
                interface.span,
            ));
        }
        if interface.shared_phenomena.is_empty() {
            errors.push(ValidationError::InterfaceWithoutPhenomena(
                interface.name.clone(),
                interface.span,
            ));
        }

        for connected in &interface.connects {
            if !defined_domains.contains(&connected.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    connected.name.clone(),
                    interface.name.clone(),
                    connected.span,
                ));
            }
        }

        for phenomenon in &interface.shared_phenomena {
            if !defined_domains.contains(&phenomenon.from.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.from.name.clone(),
                    interface.name.clone(),
                    phenomenon.from.span,
                ));
            }
            if !defined_domains.contains(&phenomenon.to.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.to.name.clone(),
                    interface.name.clone(),
                    phenomenon.to.span,
                ));
            }
            if !defined_domains.contains(&phenomenon.controlled_by.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.controlled_by.name.clone(),
                    interface.name.clone(),
                    phenomenon.controlled_by.span,
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
                ));
            }
            if phenomenon.controlled_by.name != phenomenon.from.name {
                errors.push(ValidationError::InterfaceControllerMismatch(
                    phenomenon.name.clone(),
                    interface.name.clone(),
                    phenomenon.controlled_by.name.clone(),
                    phenomenon.controlled_by.span,
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
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    for req in &problem.requirements {
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

    for req in &problem.requirements {
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
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == r.name) {
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

                        let connected_to_machine = problem.domains.iter().any(|d| {
                            d.role == DomainRole::Machine
                                && is_connected(problem, &domain.name, &d.name)
                        });

                        if !connected_to_machine {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "CommandedBehavior".to_string(),
                                req.span,
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
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == c.name) {
                        if domain.kind != DomainKind::Causal && domain.kind != DomainKind::Biddable
                        {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "RequiredBehavior".to_string(),
                                format!(
                                    "constrained domain '{}' should be causal or biddable, found {:?}/{:?}",
                                    c.name, domain.kind, domain.role
                                ),
                                c.span,
                            ));
                        }

                        let connected_to_machine = problem.domains.iter().any(|d| {
                            d.role == DomainRole::Machine
                                && is_connected(problem, &domain.name, &d.name)
                        });

                        if !connected_to_machine {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "machine".to_string(),
                                "RequiredBehavior".to_string(),
                                req.span,
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
        ValidationError::UndefinedDomainInInterface(_, _, span)
        | ValidationError::UndefinedDomainInRequirement(_, _, span)
        | ValidationError::InvalidFrameDomain(_, _, _, span)
        | ValidationError::DuplicateDomain(_, span)
        | ValidationError::DuplicateInterface(_, span)
        | ValidationError::MissingConnection(_, _, _, span)
        | ValidationError::InvalidCausality(_, _, _, _, span)
        | ValidationError::MissingRequiredField(_, _, span)
        | ValidationError::UnsupportedFrame(_, _, span)
        | ValidationError::InvalidDomainRole(_, _, span)
        | ValidationError::InterfaceInsufficientConnections(_, span)
        | ValidationError::InterfaceWithoutPhenomena(_, span)
        | ValidationError::InterfaceControllerMismatch(_, _, _, span)
        | ValidationError::RequirementReferencesMachine(_, _, span) => *span,
    }
}

fn source_path_for_error(problem: &Problem, error: &ValidationError) -> Option<PathBuf> {
    match error {
        ValidationError::UndefinedDomainInInterface(_, interface_name, _)
        | ValidationError::InterfaceInsufficientConnections(interface_name, _)
        | ValidationError::InterfaceWithoutPhenomena(interface_name, _) => problem
            .interfaces
            .iter()
            .find(|interface| interface.name == *interface_name)
            .and_then(|interface| interface.source_path.clone()),
        ValidationError::InterfaceControllerMismatch(_, interface_name, _, _) => problem
            .interfaces
            .iter()
            .find(|interface| interface.name == *interface_name)
            .and_then(|interface| interface.source_path.clone()),
        ValidationError::InvalidCausality(phenomenon_name, _, _, _, span) => problem
            .interfaces
            .iter()
            .find(|interface| {
                interface.shared_phenomena.iter().any(|phenomenon| {
                    phenomenon.name == *phenomenon_name && phenomenon.span == *span
                })
            })
            .and_then(|interface| interface.source_path.clone()),
        ValidationError::UndefinedDomainInRequirement(_, requirement_name, _)
        | ValidationError::MissingRequiredField(requirement_name, _, _)
        | ValidationError::UnsupportedFrame(requirement_name, _, _)
        | ValidationError::RequirementReferencesMachine(requirement_name, _, _) => problem
            .requirements
            .iter()
            .find(|requirement| requirement.name == *requirement_name)
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::InvalidFrameDomain(requirement_name, _, _, _) => problem
            .requirements
            .iter()
            .find(|requirement| requirement.name == *requirement_name)
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::MissingConnection(_, _, _, span) => problem
            .requirements
            .iter()
            .find(|requirement| requirement.span == *span)
            .and_then(|requirement| requirement.source_path.clone()),
        ValidationError::DuplicateDomain(domain_name, span) => problem
            .domains
            .iter()
            .find(|domain| domain.name == *domain_name && domain.span == *span)
            .and_then(|domain| domain.source_path.clone()),
        ValidationError::DuplicateInterface(interface_name, span) => problem
            .interfaces
            .iter()
            .find(|interface| interface.name == *interface_name && interface.span == *span)
            .and_then(|interface| interface.source_path.clone()),
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
