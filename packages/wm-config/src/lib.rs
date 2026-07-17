pub mod project_config;
pub mod embedding_config;
pub mod permission_preset;
pub mod permissions_config;
pub mod search_config;
pub mod recency_model;
pub mod scoring_config;
pub mod status_colors;
pub mod lsp_settings;
pub mod git_tracking;

pub use project_config::*;
pub use embedding_config::*;
pub use permission_preset::*;
pub use permissions_config::*;
pub use search_config::*;
pub use recency_model::*;
pub use scoring_config::*;
pub use status_colors::*;
pub use lsp_settings::*;
pub use git_tracking::*;

#[cfg(test)]
mod tests {
    use super::*;
    use wm_embed::SearchMode;

    #[test]
    fn test_scoring_config_defaults() {
        let cfg = ScoringConfig::default();
        assert_eq!(cfg.field_weights.get("title"), Some(&4.0));
        assert_eq!(cfg.field_weights.get("body"), Some(&1.0));
        assert_eq!(cfg.recency_model, RecencyModel::Fsrs);
        assert_eq!(cfg.recency_stability_days, 7);
        assert!((cfg.memory_salience_boost - 2.0).abs() < 1e-9);
        assert!((cfg.memory_salience_clamp - 0.1).abs() < 1e-9);
        assert_eq!(cfg.graph_depth_rrf, 1);
        assert_eq!(cfg.graph_depth_retrieve, 2);
        assert_eq!(cfg.graph_depth_retrieve_min_priority, 5);
        assert_eq!(cfg.graph_depth_neighbors_default, 2);
        assert_eq!(cfg.graph_depth_neighbors_max, 5);
        assert_eq!(cfg.debounce_ms, 500);
        assert_eq!(cfg.retrieve_token_budget, 2048);
    }

    #[test]
    fn test_search_config_defaults() {
        let cfg = SearchConfig::default();
        assert_eq!(cfg.default_mode, SearchMode::Hybrid);
        assert_eq!(cfg.default_limit, 20);
        assert_eq!(cfg.rrf_k, 60);
    }

    #[test]
    fn test_embedding_config_defaults() {
        let cfg = EmbeddingConfig::default();
        assert_eq!(cfg.model_name, "bge-small-en-v1.5");
        assert_eq!(cfg.dimensions, 384);
        assert_eq!(cfg.batch_size, 32);
    }

    #[test]
    fn test_permissions_config_defaults() {
        let cfg = PermissionsConfig::default();
        assert_eq!(cfg.preset, PermissionPreset::ReadWrite);
    }

    #[test]
    fn test_project_config_default_deserializes_from_valid_json() {
        let cfg = ProjectConfig::default();
        let json = serde_json::to_string(&cfg).expect("Serialization should succeed");
        let deserialized: ProjectConfig =
            serde_json::from_str(&json).expect("Deserialization should succeed from valid JSON");
        assert_eq!(deserialized.project_name, cfg.project_name);
        assert_eq!(deserialized.schema_version, cfg.schema_version);
        assert_eq!(
            deserialized.embedding.model_name,
            cfg.embedding.model_name
        );
        assert_eq!(deserialized.search.default_mode, cfg.search.default_mode);
        assert_eq!(
            deserialized.search.scoring.recency_model,
            cfg.search.scoring.recency_model
        );
        assert_eq!(
            deserialized.search.scoring.recency_stability_days,
            cfg.search.scoring.recency_stability_days
        );
        assert_eq!(
            deserialized.search.scoring.graph_depth_rrf,
            cfg.search.scoring.graph_depth_rrf
        );
        assert_eq!(
            deserialized.search.scoring.debounce_ms,
            cfg.search.scoring.debounce_ms
        );
    }

    #[test]
    fn test_scoring_config_optional_field_defaults() {
        let json = r#"{"field_weights": {"title": 2.0, "body": 1.0}}"#;
        let config: ScoringConfig = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(config.field_weights["title"], 2.0);
        assert_eq!(config.recency_model, RecencyModel::Fsrs);
    }

    #[test]
    fn test_permission_preset_deserialize() {
        assert_eq!(
            serde_json::from_str::<PermissionPreset>("\"read-write\"").unwrap(),
            PermissionPreset::ReadWrite
        );
        assert_eq!(
            serde_json::from_str::<PermissionPreset>("\"read-only\"").unwrap(),
            PermissionPreset::ReadOnly
        );
        assert!(serde_json::from_str::<PermissionPreset>("\"invalid\"").is_err());
    }

    #[test]
    fn test_recency_model_deserialize() {
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"fsrs\"").unwrap(),
            RecencyModel::Fsrs
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"linear\"").unwrap(),
            RecencyModel::Linear
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"exponential\"").unwrap(),
            RecencyModel::Exponential
        );
        assert_eq!(
            serde_json::from_str::<RecencyModel>("\"none\"").unwrap(),
            RecencyModel::None
        );
    }
}
