use serde::Deserialize;

use super::relation::Relation;
use super::criterion::AcceptanceCriterionFm;
use super::fr_entry::FrEntry;
use super::nfr_entry::NfrEntry;
use super::goal_entry::GoalEntry;
use super::decision_fm::DecisionFm;
use super::pattern_fm::PatternFm;
use crate::engine::TimeEntry;

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub page_type: Option<String>,
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
}
