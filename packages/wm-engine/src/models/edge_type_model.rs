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
}
