//! Edge type parsing and serde helpers for `Vec<(EdgeType, String)>` (relates_to field).

/// Custom serde module for `Vec<(EdgeType, String)>` that serializes
/// as `[{type: extends, target: "wiki:..."}]` in YAML.
pub(crate) mod relates_to_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use crate::models::edge_type_model::EdgeType;

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
                edge_type: et.as_yaml_str().to_string(),
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
            .map(|r| (EdgeType::from_str_flexible(&r.edge_type), r.target))
            .collect())
    }
}
