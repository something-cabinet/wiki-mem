//! Edge type parsing and serde helpers for `Vec<(EdgeType, String)>` (relates_to field).

use super::edge_type::EdgeType;

/// Custom serde module for `Vec<(EdgeType, String)>` that serializes
/// as `[{type: extends, target: "wiki:..."}]` in YAML.
pub(crate) mod relates_to_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::edge_type::EdgeType;

    #[derive(Serialize, Deserialize)]
    struct Relation {
        #[serde(rename = "type")]
        edge_type: String,
        target: String,
    }

    pub fn serialize<S>(val: &[(EdgeType, String)], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let items: Vec<Relation> = val
            .iter()
            .map(|(et, target)| Relation {
                edge_type: edge_type_to_yaml_str(et),
                target: target.clone(),
            })
            .collect();
        items.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<(EdgeType, String)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let items = Vec::<Relation>::deserialize(deserializer)?;
        Ok(items
            .into_iter()
            .map(|r| (super::parse_edge_type_flexible(&r.edge_type), r.target))
            .collect())
    }

    fn edge_type_to_yaml_str(et: &EdgeType) -> String {
        match et {
            EdgeType::Extends => "extends".into(),
            EdgeType::Implements => "implements".into(),
            EdgeType::ExampleOf => "example_of".into(),
            EdgeType::PartOf => "part_of".into(),
            EdgeType::RelatesTo => "relates_to".into(),
            EdgeType::Supports => "supports".into(),
            EdgeType::Contradicts => "contradicts".into(),
            EdgeType::Supersedes => "supersedes".into(),
            EdgeType::DependsOn => "depends_on".into(),
            EdgeType::RequiredBy => "required_by".into(),
            EdgeType::Questions => "questions".into(),
            EdgeType::Answers => "answers".into(),
            EdgeType::References => "references".into(),
            EdgeType::SimilarTo => "similar_to".into(),
            EdgeType::Causes => "causes".into(),
            EdgeType::Mitigates => "mitigates".into(),
            EdgeType::Custom(s) => s.clone(),
        }
    }
}

/// Parse an edge type string flexibly (supports multiple aliases).
pub fn parse_edge_type_flexible(s: &str) -> EdgeType {
    match s.to_lowercase().as_str() {
        "extends" => EdgeType::Extends,
        "implements" => EdgeType::Implements,
        "example_of" | "exampleof" | "example-of" => EdgeType::ExampleOf,
        "part_of" | "partof" | "part-of" => EdgeType::PartOf,
        "relates_to" | "relates-to" | "relatesto" | "related" => EdgeType::RelatesTo,
        "supports" => EdgeType::Supports,
        "contradicts" => EdgeType::Contradicts,
        "supersedes" => EdgeType::Supersedes,
        "depends_on" | "dependson" | "depends-on" => EdgeType::DependsOn,
        "required_by" | "requiredby" | "required-by" => EdgeType::RequiredBy,
        "questions" => EdgeType::Questions,
        "answers" => EdgeType::Answers,
        "references" => EdgeType::References,
        "similar_to" | "similarto" | "similar-to" | "similar" => EdgeType::SimilarTo,
        "causes" => EdgeType::Causes,
        "mitigates" => EdgeType::Mitigates,
        custom => EdgeType::Custom(custom.to_string()),
    }
}
