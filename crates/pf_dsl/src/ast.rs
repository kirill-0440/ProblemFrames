#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub name: String,
    pub span: Span,
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
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Domain {
    pub name: String,
    pub domain_type: DomainType,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Interface {
    pub name: String,
    pub shared_phenomena: Vec<Phenomenon>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhenomenonType {
    Event,
    State,
    Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phenomenon {
    pub name: String,
    pub type_: PhenomenonType,
    pub from: String,
    pub to: String,
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
    pub phenomena: Vec<String>, // Simplification for now
    pub constraint: String,
    pub constrains: String,
    pub reference: String,
    pub span: Span,
}
