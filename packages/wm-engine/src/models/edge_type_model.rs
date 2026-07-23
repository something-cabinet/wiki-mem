use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
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
            EdgeType::Implements => 9,
            EdgeType::PartOf => 8,
            EdgeType::Supersedes => 8,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn => 5,
            EdgeType::Answers => 5,
            EdgeType::References => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
        }
    }

    /// Canonical YAML string representation (e.g., `EdgeType::Extends` → `"extends"`).
    pub fn as_yaml_str(&self) -> &str {
        match self {
            EdgeType::Extends => "extends",
            EdgeType::Implements => "implements",
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

    /// Flexible parser supporting multiple aliases (kebab, snake, compound words).
    /// Unknown strings produce `EdgeType::Custom(input)`.
    pub fn from_str_flexible(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extends" => EdgeType::Extends,
            "implements" => EdgeType::Implements,
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
