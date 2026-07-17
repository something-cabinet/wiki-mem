use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::status::{Confidence, PageStatus, Priority};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    #[serde(with = "crate::engine::relation::relates_to_vec")]
    pub relates_to: Vec<(crate::engine::EdgeType, String)>,
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
    pub page_type: crate::engine::PageType,
    pub order: Option<i32>,
    pub task_data: Option<crate::engine::TaskData>,
    pub spec_data: Option<crate::engine::SpecData>,
    pub decision_data: Option<crate::engine::DecisionData>,
    pub pattern_data: Option<crate::engine::PatternData>,
    pub memory_data: Option<crate::engine::MemoryData>,
    pub rule_data: Option<crate::engine::RuleData>,
}
