use crate::ast::*;
use std::collections::HashSet;
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
    #[error("Invalid causality: Phenomenon '{0}' ({1:?}) cannot originate from '{2}' ({3:?}). Events must come from active domains.")]
    InvalidCausality(String, PhenomenonType, String, DomainType, Span),
    #[error("Requirement '{0}' is missing required field '{1}'.")]
    MissingRequiredField(String, String, Span),
    #[error("Requirement '{0}' uses unsupported frame '{1}'.")]
    UnsupportedFrame(String, String, Span),
}

fn is_connected(problem: &Problem, domain1: &str, domain2: &str) -> bool {
    problem.interfaces.iter().any(|i| {
        i.shared_phenomena.iter().any(|p| {
            (p.from.name == domain1 && p.to.name == domain2)
                || (p.from.name == domain2 && p.to.name == domain1)
        })
    })
}

pub fn validate(problem: &Problem) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    let mut defined_domains = HashSet::new();

    // 0. check for duplicates
    for domain in &problem.domains {
        if defined_domains.contains(&domain.name) {
            errors.push(ValidationError::DuplicateDomain(
                domain.name.clone(),
                domain.span,
            ));
        } else {
            defined_domains.insert(domain.name.clone());
        }
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

    // 1. Validate Interfaces
    for interface in &problem.interfaces {
        for phenomenon in &interface.shared_phenomena {
            // Check existence of domains
            if !defined_domains.contains(&phenomenon.from.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.from.name.clone(),
                    interface.name.clone(),
                    phenomenon.from.span, // Precise span
                ));
            }
            if !defined_domains.contains(&phenomenon.to.name) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.to.name.clone(),
                    interface.name.clone(),
                    phenomenon.to.span, // Precise span
                ));
            }

            // CAUSALITY CHECKS
            if let Some(from_domain) = problem
                .domains
                .iter()
                .find(|d| d.name == phenomenon.from.name)
            {
                match phenomenon.type_ {
                    PhenomenonType::Event | PhenomenonType::Command => {
                        // 1. Active Domain Check
                        // Events/Commands cannot originate from Lexical or Designed domains (inert)
                        if from_domain.domain_type == DomainType::Lexical
                            || from_domain.domain_type == DomainType::Designed
                        {
                            errors.push(ValidationError::InvalidCausality(
                                phenomenon.name.clone(),
                                phenomenon.type_.clone(),
                                from_domain.name.clone(),
                                from_domain.domain_type.clone(),
                                phenomenon.span,
                            ));
                        }

                        // 2. Operator Command Check
                        if phenomenon.type_ == PhenomenonType::Command
                            && from_domain.domain_type != DomainType::Biddable
                        {
                            errors.push(ValidationError::InvalidCausality(
                                phenomenon.name.clone(),
                                phenomenon.type_.clone(), // "Command"
                                from_domain.name.clone(),
                                from_domain.domain_type.clone(),
                                phenomenon.span,
                            ));
                        }
                    }
                    _ => {} // State/Value can exist/originate anywhere
                }
            }
        }
    }

    // 2. Validate Requirements
    for req in &problem.requirements {
        if let Some(ref c) = req.constrains {
            if !defined_domains.contains(&c.name) {
                errors.push(ValidationError::UndefinedDomainInRequirement(
                    c.name.clone(),
                    req.name.clone(),
                    c.span, // Precise span
                ));
            }
        }

        if let Some(ref r) = req.reference {
            if !defined_domains.contains(&r.name) {
                errors.push(ValidationError::UndefinedDomainInRequirement(
                    r.name.clone(),
                    req.name.clone(),
                    r.span, // Precise span
                ));
            }
        }
    }

    // 3. Validate Frame Constraints
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

                // 1. Reference domain (Operator) must be Biddable
                if let Some(ref r) = req.reference {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == r.name) {
                        if domain.domain_type != DomainType::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "CommandedBehavior".to_string(),
                                format!(
                                    "Reference domain '{}' should be Biddable (Operator), found {:?}",
                                    r.name, domain.domain_type
                                ),
                                r.span,
                            ));
                        }

                        // 2. Topology: Operator -> Machine
                        let connected_to_machine = problem.domains.iter().any(|d| {
                            d.domain_type == DomainType::Machine
                                && is_connected(problem, &domain.name, &d.name)
                        });

                        if !connected_to_machine {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "any Machine".to_string(),
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

                // 1. Constrained domain must be Causal or Biddable
                if let Some(ref c) = req.constrains {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == c.name) {
                        if domain.domain_type != DomainType::Causal
                            && domain.domain_type != DomainType::Biddable
                        {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "RequiredBehavior".to_string(),
                                format!("Constrained domain '{}' should be Causal or Biddable, found {:?}", c.name, domain.domain_type),
                                c.span,
                            ));
                        }

                        // 2. Topology: Machine -> Constrained Domain
                        let connected_to_machine = problem.domains.iter().any(|d| {
                            d.domain_type == DomainType::Machine
                                && is_connected(problem, &domain.name, &d.name)
                        });

                        if !connected_to_machine {
                            errors.push(ValidationError::MissingConnection(
                                domain.name.clone(),
                                "any Machine".to_string(),
                                "RequiredBehavior".to_string(),
                                req.span,
                            ));
                        }
                    }
                }
            }
            _ => {} // Other frames to be implemented
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
