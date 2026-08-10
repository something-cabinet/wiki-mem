use serde::Deserialize;

use super::criterion_model::AcceptanceCriterionFm;
use super::decision_fm_model::DecisionFm;
use super::fr_entry_model::FrEntry;
use super::goal_entry_model::GoalEntry;
use super::nfr_entry_model::NfrEntry;
use super::pattern_fm_model::PatternFm;
use super::relation_model::Relation;
use crate::shared::traits::Parser;
use wm_engine::RuleCategory;
use wm_engine::TimeEntry;

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub page_type: Option<String>,
    /// Any frontmatter key that isn't a modeled field (custom fields such as
    /// createdAt/updatedAt, user-defined keys, ...) is captured here so a
    /// struct round-trip through `frontmatter_to_yaml` never drops it.
    #[serde(flatten)]
    pub unknown: std::collections::BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub relates_to: Vec<Relation>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub confidence: Option<String>,
    #[serde(default)]
    pub superseded_by: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterionFm>,
    #[serde(default)]
    pub estimate: Option<u32>,
    #[serde(default)]
    pub functional_requirements: Vec<FrEntry>,
    #[serde(default)]
    pub non_functional_requirements: Vec<NfrEntry>,
    #[serde(default)]
    pub general_goals: Vec<GoalEntry>,
    #[serde(default)]
    pub stakeholders: Vec<String>,
    #[serde(default)]
    pub decision: Option<DecisionFm>,
    #[serde(default)]
    pub pattern: Option<PatternFm>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub difficulty: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub time_started: Option<String>,
    #[serde(default)]
    pub time_spent: Option<String>,
    #[serde(default)]
    pub time_entries: Option<Vec<TimeEntry>>,
    #[serde(default)]
    pub order: Option<i32>,
    #[serde(default)]
    pub implementation_plan: Option<String>,
    #[serde(default)]
    pub implementation_notes: Option<String>,
    #[serde(default)]
    pub category: Option<RuleCategory>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub example: Option<String>,
    #[serde(default)]
    pub anti_pattern: Option<String>,
}

impl Parser<Self> for Frontmatter {
    fn parse(input: &str) -> Result<Self, crate::error::ToolError> {
        let (fm, _body) = crate::parser::extract_frontmatter(input);
        fm.ok_or_else(|| crate::error::ToolError::internal("No frontmatter found in content"))
    }
}
