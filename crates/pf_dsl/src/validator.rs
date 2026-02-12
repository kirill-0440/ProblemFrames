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
}

pub fn validate(problem: &Problem) -> Result<(), Vec<ValidationError>> {
    let mut errors = vec![];
    let defined_domains: HashSet<&String> = problem.domains.iter().map(|d| &d.name).collect();

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
                // Should have a Biddable domain as reference (Operator) and Machine as implicit/explicit
                if !req.reference.is_empty() {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == req.reference) {
                        if domain.domain_type != DomainType::Biddable {
                             errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "CommandedBehavior".to_string(),
                                format!("Reference domain '{}' should be Biddable, found {:?}", req.reference, domain.domain_type),
                                req.span,
                            ));
                        }
                    }
                }
            },
            FrameType::RequiredBehavior => {
                // Should constrain a Causal or Biddable domain
                if !req.constrains.is_empty() {
                    if let Some(domain) = problem.domains.iter().find(|d| d.name == req.constrains) {
                         if domain.domain_type != DomainType::Causal && domain.domain_type != DomainType::Biddable {
                             errors.push(ValidationError::InvalidFrameDomain(
                                req.name.clone(),
                                "RequiredBehavior".to_string(),
                                format!("Constrained domain '{}' should be Causal or Biddable, found {:?}", req.constrains, domain.domain_type),
                                req.span,
                            ));
                         }
                    }
                }
            },
            _ => {} // Other frames to be implemented
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
