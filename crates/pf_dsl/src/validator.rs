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
}

fn is_connected(problem: &Problem, domain1: &str, domain2: &str) -> bool {
    problem.interfaces.iter().any(|i| {
        i.shared_phenomena.iter().any(|p| {
            (p.from == domain1 && p.to == domain2) || (p.from == domain2 && p.to == domain1)
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
            if !defined_domains.contains(&phenomenon.from) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.from.clone(),
                    interface.name.clone(),
                    phenomenon.span, // Use phenomenon span for precise error
                ));
            }
            if !defined_domains.contains(&phenomenon.to) {
                errors.push(ValidationError::UndefinedDomainInInterface(
                    phenomenon.to.clone(),
                    interface.name.clone(),
                    phenomenon.span,
                ));
            }
        }
    }

    // 2. Validate Requirements
    for req in &problem.requirements {
        if !req.constrains.is_empty() && !defined_domains.contains(&req.constrains) {
            errors.push(ValidationError::UndefinedDomainInRequirement(
                req.constrains.clone(),
                req.name.clone(),
                req.span, // We could be more precise if we parsed specific fields with spans
            ));
        }
        if !req.reference.is_empty() && !defined_domains.contains(&req.reference) {
            errors.push(ValidationError::UndefinedDomainInRequirement(
                req.reference.clone(),
                req.name.clone(),
                req.span,
            ));
        }
    }

    // 3. Validate Frame Constraints
    for req in &problem.requirements {
        match req.frame {
            FrameType::CommandedBehavior => {
                // 1. Reference domain (Operator) must be Biddable
                if !req.reference.is_empty() {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == req.reference) {
                        if domain.domain_type != DomainType::Biddable {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "CommandedBehavior".to_string(),
                                format!(
                                    "Reference domain '{}' should be Biddable (Operator), found {:?}",
                                    req.reference, domain.domain_type
                                ),
                                req.span,
                            ));
                        }
                    }
                }

                // 2. Topology: Operator -> Machine
                // We need to find the Machine. It's usually the other domain not mentioned in reference/constrains,
                // OR we just check if *some* machine refers to the referenced Operator.
                // For simplified v1: We check if Reference (Operator) is connected to ANY Machine.
                // Improve later: identify the specific Machine involved in this Requirement.
                if let Some(operator) = problem.domains.iter().find(|d| d.name == req.reference) {
                    let connected_to_machine = problem.domains.iter().any(|d| {
                        d.domain_type == DomainType::Machine
                            && is_connected(problem, &operator.name, &d.name)
                    });

                    if !connected_to_machine {
                        errors.push(ValidationError::MissingConnection(
                            operator.name.clone(),
                            "any Machine".to_string(),
                            "CommandedBehavior".to_string(),
                            req.span,
                        ));
                    }
                }
            }
            FrameType::RequiredBehavior => {
                // 1. Constrained domain must be Causal or Biddable
                if !req.constrains.is_empty() {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == req.constrains)
                    {
                        if domain.domain_type != DomainType::Causal
                            && domain.domain_type != DomainType::Biddable
                        {
                            errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "RequiredBehavior".to_string(),
                                format!("Constrained domain '{}' should be Causal or Biddable, found {:?}", req.constrains, domain.domain_type),
                                req.span,
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
