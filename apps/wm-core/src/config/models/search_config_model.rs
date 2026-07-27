use super::scoring_config_model::ScoringConfig;
use serde::{Deserialize, Serialize};
use wm_embed::SearchMode;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub default_mode: SearchMode,
    pub default_limit: u32,
    pub rrf_k: u32,
    #[serde(default)]
    pub scoring: ScoringConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_mode: SearchMode::Hybrid,
            default_limit: 20,
            rrf_k: 60,
            scoring: ScoringConfig::default(),
        }
    }
}
