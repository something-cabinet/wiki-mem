use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::helpers::relation_helper::relates_to_vec;
use crate::models::edge_type_model::EdgeType;
use crate::models::page_data::{
    DecisionData, MemoryData, PatternData, RuleData, SpecData, TaskData,
};
use crate::models::page_type_model::PageType;
use crate::status::{Confidence, PageStatus, Priority};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub status: PageStatus,
    pub published: bool,
    pub priority: Option<Priority>,
    pub confidence: Option<Confidence>,
    pub assignee: Option<String>,
    pub aliases: Vec<String>,
    pub superseded_by: Option<String>,
    pub version: Option<String>,
    pub sources: Vec<String>,
    pub parent: Option<String>,
    #[serde(with = "relates_to_vec")]
    pub relates_to: Vec<(EdgeType, String)>,
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
    pub page_type: PageType,
    pub order: Option<i32>,
    pub task_data: Option<TaskData>,
    pub spec_data: Option<SpecData>,
    pub decision_data: Option<DecisionData>,
    pub pattern_data: Option<PatternData>,
    pub memory_data: Option<MemoryData>,
    pub rule_data: Option<RuleData>,
    #[serde(default, skip_serializing)]
    pub raw_type: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_status: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_priority: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_confidence: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_relates_to: Vec<(String, String)>,
    #[serde(default, skip_serializing)]
    pub raw_rationale: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_example: Option<String>,
    #[serde(default, skip_serializing)]
    pub raw_anti_pattern: Option<String>,
    #[serde(default, skip_serializing)]
    pub time_started: Option<String>,
    #[serde(default, skip_serializing)]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing)]
    pub unknown: BTreeMap<String, serde_yaml::Value>,
}
