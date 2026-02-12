use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Reference {
    pub name: String,
    pub span: Span,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub name: String,
    pub span: Span,
    pub imports: Vec<String>,
    pub domains: Vec<Domain>,
    pub interfaces: Vec<Interface>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DomainType {
    Machine,
    Causal,
    Biddable,
    Lexical,
    Designed,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Domain {
    pub name: String,
    pub domain_type: DomainType,
    pub span: Span,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub name: String,
    pub shared_phenomena: Vec<Phenomenon>,
    pub span: Span,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhenomenonType {
    Event,
    Command,
    State,
    Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phenomenon {
    pub name: String,
    pub type_: PhenomenonType,
    pub from: Reference,
    pub to: Reference,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrameType {
    RequiredBehavior,
    CommandedBehavior,
    InformationDisplay,
    SimpleWorkpieces,
    Transformation,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Requirement {
    pub name: String,
    pub frame: FrameType,
    pub phenomena: Vec<String>,
    // constraint is just text, not a reference to a domain
    pub constraint: String,
    // these refer to domains
    pub constrains: Option<Reference>,
    pub reference: Option<Reference>,
    pub span: Span,
    pub source_path: Option<PathBuf>,
}
