use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::recency_model::RecencyModel;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_field_weights")]
    pub field_weights: HashMap<String, f64>,
    #[serde(default)]
    pub recency_model: RecencyModel,
    #[serde(default = "default_recency_stability_days")]
    pub recency_stability_days: u32,
    #[serde(default = "default_memory_salience_boost")]
    pub memory_salience_boost: f64,
    #[serde(default = "default_memory_salience_clamp")]
    pub memory_salience_clamp: f64,
    #[serde(default = "default_graph_depth_rrf")]
    pub graph_depth_rrf: u32,
    #[serde(default = "default_graph_depth_retrieve")]
    pub graph_depth_retrieve: u32,
    #[serde(default = "default_graph_depth_retrieve_min_priority")]
    pub graph_depth_retrieve_min_priority: u8,
    #[serde(default = "default_graph_depth_neighbors_default")]
    pub graph_depth_neighbors_default: u32,
    #[serde(default = "default_graph_depth_neighbors_max")]
    pub graph_depth_neighbors_max: u32,
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,
    #[serde(default = "default_retrieve_token_budget")]
    pub retrieve_token_budget: usize,
}

fn default_field_weights() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("title".into(), 4.0);
    m.insert("body".into(), 1.0);
    m
}
fn default_recency_stability_days() -> u32 { 7 }
fn default_memory_salience_boost() -> f64 { 2.0 }
fn default_memory_salience_clamp() -> f64 { 0.1 }
fn default_graph_depth_rrf() -> u32 { 1 }
fn default_graph_depth_retrieve() -> u32 { 2 }
fn default_graph_depth_retrieve_min_priority() -> u8 { 5 }
fn default_graph_depth_neighbors_default() -> u32 { 2 }
fn default_graph_depth_neighbors_max() -> u32 { 5 }
fn default_debounce_ms() -> u64 { 500 }
fn default_retrieve_token_budget() -> usize { 2048 }

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            field_weights: default_field_weights(),
            recency_model: RecencyModel::default(),
            recency_stability_days: default_recency_stability_days(),
            memory_salience_boost: default_memory_salience_boost(),
            memory_salience_clamp: default_memory_salience_clamp(),
            graph_depth_rrf: default_graph_depth_rrf(),
            graph_depth_retrieve: default_graph_depth_retrieve(),
            graph_depth_retrieve_min_priority: default_graph_depth_retrieve_min_priority(),
            graph_depth_neighbors_default: default_graph_depth_neighbors_default(),
            graph_depth_neighbors_max: default_graph_depth_neighbors_max(),
            debounce_ms: default_debounce_ms(),
            retrieve_token_budget: default_retrieve_token_budget(),
        }
    }
}
