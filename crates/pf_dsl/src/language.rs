use crate::ast::{DomainType, FrameType, PhenomenonType};

pub const STATEMENT_KEYWORDS: &[&str] =
    &["problem:", "domain", "interface", "requirement", "shared:"];

pub const REQUIREMENT_FIELDS: &[&str] = &["frame:", "constraint:", "constrains:", "reference:"];

pub const DOMAIN_TYPES: &[&str] = &["Machine", "Causal", "Biddable", "Lexical", "Designed"];

pub const PHENOMENON_TYPES: &[&str] = &["event", "command", "state", "value"];

pub const FRAME_TYPES: &[&str] = &[
    "RequiredBehavior",
    "CommandedBehavior",
    "InformationDisplay",
    "SimpleWorkpieces",
    "Transformation",
];

pub fn parse_domain_type(value: &str) -> DomainType {
    match value {
        "Machine" => DomainType::Machine,
        "Causal" => DomainType::Causal,
        "Biddable" => DomainType::Biddable,
        "Lexical" => DomainType::Lexical,
        "Designed" => DomainType::Designed,
        _ => DomainType::Unknown(value.to_string()),
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
