use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
    ExampleOf,
    PartOf,
    RelatesTo,
    Supports,
    Contradicts,
    Supersedes,
    DependsOn,
    RequiredBy,
    Questions,
    Answers,
    References,
    SimilarTo,
    Causes,
    Mitigates,
    Custom(String),
}

impl EdgeType {
    pub fn priority(&self) -> u8 {
        match self {
            EdgeType::Extends => 10,
            EdgeType::Implements => 9,
            EdgeType::PartOf => 8,
            EdgeType::Supports => 7,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn | EdgeType::RequiredBy => 5,
            EdgeType::Mitigates | EdgeType::Causes => 4,
            EdgeType::Contradicts | EdgeType::Questions => 3,
            EdgeType::Answers => 2,
            EdgeType::References | EdgeType::SimilarTo => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
            EdgeType::Supersedes => 8,
        }
    }
}
