use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum EdgeProvenance {
    Explicit,
    Derived,
    Ambiguous,
}

impl EdgeProvenance {
    pub const EXPLICIT_FACTOR: f64 = 1.0;
    pub const DERIVED_FACTOR: f64 = 0.5;
    pub const AMBIGUOUS_FACTOR: f64 = 0.25;

    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeProvenance::Explicit => "explicit",
            EdgeProvenance::Derived => "derived",
            EdgeProvenance::Ambiguous => "ambiguous",
        }
    }

    pub fn factor(&self) -> f64 {
        match self {
            EdgeProvenance::Explicit => Self::EXPLICIT_FACTOR,
            EdgeProvenance::Derived => Self::DERIVED_FACTOR,
            EdgeProvenance::Ambiguous => Self::AMBIGUOUS_FACTOR,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphEdge {
    pub edge_type: EdgeType,
    pub provenance: EdgeProvenance,
}

impl GraphEdge {
    pub fn new(edge_type: EdgeType, provenance: EdgeProvenance) -> Self {
        Self {
            edge_type,
            provenance,
        }
    }

    pub fn priority(&self) -> u8 {
        self.edge_type.priority()
    }

    pub fn provenance_factor(&self) -> f64 {
        self.provenance.factor()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
    Calls,
    Inherits,
    Imports,
    ExampleOf,
    PartOf,
    RelatesTo,
    Supersedes,
    DependsOn,
    Answers,
    References,
    Custom(String),
}

impl EdgeType {
    pub fn priority(&self) -> u8 {
        match self {
            EdgeType::Extends => 10,
            EdgeType::Calls | EdgeType::Inherits => 9,
            EdgeType::Implements => 9,
            EdgeType::Imports => 7,
            EdgeType::PartOf => 8,
            EdgeType::Supersedes => 8,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn => 5,
            EdgeType::Answers => 5,
            EdgeType::References => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
        }
    }

    pub fn as_yaml_str(&self) -> &str {
        match self {
            EdgeType::Extends => "extends",
            EdgeType::Implements => "implements",
            EdgeType::Calls => "calls",
            EdgeType::Inherits => "inherits",
            EdgeType::Imports => "imports",
            EdgeType::ExampleOf => "example_of",
            EdgeType::PartOf => "part_of",
            EdgeType::RelatesTo => "relates_to",
            EdgeType::Supersedes => "supersedes",
            EdgeType::DependsOn => "depends_on",
            EdgeType::Answers => "answers",
            EdgeType::References => "references",
            EdgeType::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str_flexible(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extends" => EdgeType::Extends,
            "implements" => EdgeType::Implements,
            "calls" => EdgeType::Calls,
            "inherits" => EdgeType::Inherits,
            "imports" | "import" => EdgeType::Imports,
            "example_of" | "exampleof" | "example-of" => EdgeType::ExampleOf,
            "part_of" | "partof" | "part-of" => EdgeType::PartOf,
            "relates_to" | "relates-to" | "relatesto" | "related" => EdgeType::RelatesTo,
            "supersedes" => EdgeType::Supersedes,
            "depends_on" | "dependson" | "depends-on" => EdgeType::DependsOn,
            "answers" => EdgeType::Answers,
            "references" => EdgeType::References,
            custom => EdgeType::Custom(custom.to_string()),
        }
    }
}
