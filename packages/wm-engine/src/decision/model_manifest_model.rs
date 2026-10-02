use serde::Deserialize;

use super::decision_error_model::DecisionError;
use super::model_entry_model::ModelEntry;

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
const HF_SOURCE_PREFIX: &str = "hf:";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelManifest {
    pub schema_version: u32,
    pub models: Vec<ModelEntry>,
}

impl ModelManifest {
    pub fn parse(json: &str) -> Result<ModelManifest, DecisionError> {
        let manifest: ModelManifest = serde_json::from_str(json)
            .map_err(|error| DecisionError::InvalidManifest(error.to_string()))?;
        if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(DecisionError::InvalidManifest(format!(
                "unsupported schema_version {}; expected {}",
                manifest.schema_version, MANIFEST_SCHEMA_VERSION
            )));
        }
        if manifest.models.is_empty() {
            return Err(DecisionError::InvalidManifest(
                "manifest declares no models".to_owned(),
            ));
        }
        for entry in &manifest.models {
            validate_entry(entry)?;
        }
        Ok(manifest)
    }

    pub fn entry(&self, name: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|entry| entry.name == name)
    }
}

fn validate_entry(entry: &ModelEntry) -> Result<(), DecisionError> {
    if !entry.source.starts_with(HF_SOURCE_PREFIX) {
        return Err(DecisionError::InvalidManifest(format!(
            "model '{}' source must start with {HF_SOURCE_PREFIX}",
            entry.name
        )));
    }
    if entry.revision.is_empty() {
        return Err(DecisionError::InvalidManifest(format!(
            "model '{}' has no pinned revision",
            entry.name
        )));
    }
    if entry.files.is_empty() {
        return Err(DecisionError::InvalidManifest(format!(
            "model '{}' declares no files",
            entry.name
        )));
    }
    for file in &entry.files {
        if !file.has_valid_hash() {
            return Err(DecisionError::InvalidManifest(format!(
                "model '{}' file '{}' has an invalid sha256",
                entry.name, file.path
            )));
        }
    }
    Ok(())
}

pub fn source_repo(source: &str) -> Option<&str> {
    source.strip_prefix(HF_SOURCE_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{
      "schema_version": 1,
      "models": [
        {
          "name": "gliner2.5-small-v1",
          "source": "hf:fastino/gliner2.5-small-v1",
          "revision": "7132dc4561c3f94563c6147e75ffa8ef34c4964a",
          "license": "apache-2.0",
          "max_input_tokens": 512,
          "files": [
            {
              "path": "model.safetensors",
              "sha256": "4ee982787ace270d4bf15dbcb28ced38e0aa201372347114ceedd6336055de2b",
              "size_bytes": 295567700
            }
          ]
        }
      ]
    }"#;

    #[test]
    fn parses_a_valid_manifest_and_finds_entries() {
        let manifest = ModelManifest::parse(MANIFEST).expect("manifest should parse");
        assert_eq!(manifest.schema_version, MANIFEST_SCHEMA_VERSION);
        let entry = manifest.entry("gliner2.5-small-v1").expect("entry present");
        assert_eq!(entry.max_input_tokens, 512);
        assert_eq!(entry.license, "apache-2.0");
        assert_eq!(
            source_repo(&entry.source),
            Some("fastino/gliner2.5-small-v1")
        );
        assert!(manifest.entry("missing").is_none());
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let json = MANIFEST.replace("\"schema_version\": 1", "\"schema_version\": 2");
        assert!(matches!(
            ModelManifest::parse(&json),
            Err(DecisionError::InvalidManifest(_))
        ));
    }

    #[test]
    fn rejects_invalid_digest() {
        let json = MANIFEST.replace(
            "4ee982787ace270d4bf15dbcb28ced38e0aa201372347114ceedd6336055de2b",
            "nope",
        );
        assert!(matches!(
            ModelManifest::parse(&json),
            Err(DecisionError::InvalidManifest(_))
        ));
    }

    #[test]
    fn accepts_placeholder_zero_hash() {
        let json = MANIFEST.replace(
            "4ee982787ace270d4bf15dbcb28ced38e0aa201372347114ceedd6336055de2b",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        let manifest = ModelManifest::parse(&json).expect("placeholder manifest should parse");
        assert!(manifest.models[0].files[0].has_placeholder_hash());
    }

    #[test]
    fn rejects_missing_revision() {
        let json = MANIFEST.replace("7132dc4561c3f94563c6147e75ffa8ef34c4964a", "");
        assert!(matches!(
            ModelManifest::parse(&json),
            Err(DecisionError::InvalidManifest(_))
        ));
    }

    #[test]
    fn reads_the_committed_models_manifest() {
        let json = include_str!("../../../../.wm/models.json");
        let manifest = ModelManifest::parse(json).expect("committed .wm/models.json must parse");
        let entry = manifest
            .entry("gliner2.5-small-v1")
            .expect("committed manifest must declare gliner2.5-small-v1");
        assert_eq!(entry.max_input_tokens, 512);
        assert_eq!(entry.revision, "7132dc4561c3f94563c6147e75ffa8ef34c4964a");
    }
}
