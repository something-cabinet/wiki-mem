use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wm_status::{Confidence, PageStatus, Priority};

use crate::models::edge_type_model::EdgeType;
use crate::models::page_type_model::PageType;
use crate::models::page_data::{TaskData, SpecData, DecisionData, PatternData, MemoryData, RuleData};
use crate::helpers::relation_helper::relates_to_vec;

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
}
