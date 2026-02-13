use crate::ast::{DomainKind, DomainRole, FrameType, PhenomenonType};

pub const STATEMENT_KEYWORDS: &[&str] = &[
    "problem:",
    "domain",
    "kind",
    "role",
    "interface",
    "connects",
    "phenomenon",
    "controlledBy",
    "requirement",
    "shared:",
];

pub const REQUIREMENT_FIELDS: &[&str] = &["frame:", "constraint:", "constrains:", "reference:"];

pub const DOMAIN_KINDS: &[&str] = &["biddable", "causal", "lexical"];

pub const DOMAIN_ROLES: &[&str] = &["given", "designed", "machine"];

pub const PHENOMENON_TYPES: &[&str] = &["event", "command", "state", "value"];

pub const FRAME_TYPES: &[&str] = &[
    "RequiredBehavior",
    "CommandedBehavior",
    "InformationDisplay",
    "SimpleWorkpieces",
    "Transformation",
];

pub fn parse_domain_kind(value: &str) -> DomainKind {
    match value {
        "biddable" => DomainKind::Biddable,
        "causal" => DomainKind::Causal,
        "lexical" => DomainKind::Lexical,
        _ => DomainKind::Unknown(value.to_string()),
    }
}

pub fn parse_domain_role(value: &str) -> DomainRole {
    match value {
        "given" => DomainRole::Given,
        "designed" => DomainRole::Designed,
        "machine" => DomainRole::Machine,
        _ => DomainRole::Unknown(value.to_string()),
    }
}

pub fn parse_phenomenon_type(value: &str) -> Option<PhenomenonType> {
    match value {
        "event" => Some(PhenomenonType::Event),
        "command" => Some(PhenomenonType::Command),
        "state" => Some(PhenomenonType::State),
        "value" => Some(PhenomenonType::Value),
        _ => None,
    }
}

pub fn parse_frame_type(value: &str) -> FrameType {
    match value {
        "RequiredBehavior" => FrameType::RequiredBehavior,
        "CommandedBehavior" => FrameType::CommandedBehavior,
        "InformationDisplay" => FrameType::InformationDisplay,
        "SimpleWorkpieces" => FrameType::SimpleWorkpieces,
        "Transformation" => FrameType::Transformation,
        _ => FrameType::Custom(value.to_string()),
    }
}
