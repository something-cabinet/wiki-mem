use arc_swap::ArcSwap;
use dashmap::DashMap;
use petgraph::stable_graph::StableGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::config::ProjectConfig;
use crate::search::Bm25Index;

// ─── Edge Types ─────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Extends,
    Implements,
    ExampleOf,
    PartOf,
    RelatesTo,
    Supports,
    Contradicts,
    Supersedes,
    DependsOn,
    RequiredBy,
    Questions,
    Answers,
    References,
    SimilarTo,
    Causes,
    Mitigates,
    Custom(&'static str),
}

impl EdgeType {
    pub fn priority(&self) -> u8 {
        match self {
            EdgeType::Extends => 10,
            EdgeType::Implements => 9,
            EdgeType::PartOf => 8,
            EdgeType::Supports => 7,
            EdgeType::ExampleOf => 6,
            EdgeType::DependsOn | EdgeType::RequiredBy => 5,
            EdgeType::Mitigates | EdgeType::Causes => 4,
            EdgeType::Contradicts | EdgeType::Questions => 3,
            EdgeType::Answers => 2,
            EdgeType::References | EdgeType::SimilarTo => 1,
            EdgeType::RelatesTo | EdgeType::Custom(_) => 0,
            EdgeType::Supersedes => 8,
        }
    }
}

// ─── Page Types ─────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task, Spec, Concept, Pattern, Decision, Howto, Reference,
}

// ─── Page Status ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PageStatus {
    Todo, InProgress, Done, Blocked, Cancelled,
    Draft, Reviewed, Superseded, Approved,
}

// ─── Priorities & Confidence ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority { Low, Medium, High, Urgent }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confidence { High, Medium, Low }

// ─── Spec-specific fields ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralGoal {
    pub description: String,
}

// ─── Decision-specific fields ───────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecisionEntry {
    pub context: String,
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
}

// ─── Pattern-specific fields ────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternInfo {
    pub when_to_use: String,
    pub example: String,
}

// ─── Acceptance Criterion ───────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub text: String,
    pub checked: bool,
}

// ─── Wiki Page Metadata ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub id: String,
    pub title: String,
    pub page_type: PageType,
    pub tags: Vec<String>,
    pub status: PageStatus,
    pub priority: Option<Priority>,
    pub confidence: Option<Confidence>,
    pub assignee: Option<String>,
    pub aliases: Vec<String>,
    pub superseded_by: Option<String>,
    pub version: Option<String>,
    pub sources: Vec<String>,
    // Per-type structured fields
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub estimate: Option<u32>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub general_goals: Vec<GeneralGoal>,
    pub stakeholders: Vec<String>,
    pub decision: Option<DecisionEntry>,
    pub pattern: Option<PatternInfo>,
    pub prerequisites: Vec<String>,
    pub difficulty: Option<String>,
    pub source_url: Option<String>,
    // Relationships as string list
    pub relates_to: Vec<String>,
    // Path & timestamps
    pub path: PathBuf,
    pub created_at: String,
    pub updated_at: String,
}

// ─── Wiki Page Content ──────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WikiPageContent {
    pub raw: String,
    pub sections: Vec<SectionDoc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SectionDoc {
    pub section_id: String,
    pub page_id: String,
    pub header: String,
    pub body: String,
}

// ─── Source Entry ───────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SourceState {
    Pending, Processing, Done, Error, Stale,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub original_path: Option<String>,
    pub stored_path: PathBuf,
    pub content_hash: String,
    pub state: SourceState,
    pub added_at: String,
    pub last_processed_at: Option<String>,
    pub page_refs: Vec<String>,
    pub page_count: usize,
    pub error_message: Option<String>,
    pub retry_count: usize,
}

// ─── Audit Event ────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub tool_name: String,
    pub action: String,
    pub duration_ms: i64,
    pub result: String,
    pub error_message: Option<String>,
    pub entity_refs: Vec<String>,
}

// ─── Graph Snapshot ─────────────────────────────────────────

pub type GraphSnapshot = (petgraph::stable_graph::StableGraph<WikiPageMeta, EdgeType>, HashMap<String, petgraph::stable_graph::NodeIndex>);

// ─── Engine State ───────────────────────────────────────────

pub struct EngineState {
    pub graph: ArcSwap<GraphSnapshot>,
    pub page_contents: DashMap<String, WikiPageContent>,
    pub section_corpus: ArcSwap<Vec<SectionDoc>>,
    pub bm25_index: ArcSwap<Bm25Index>,
    pub vector_registry: ArcSwap<HashMap<String, Vec<f32>>>,
    pub source_registry: RwLock<HashMap<String, SourceEntry>>,
    pub content_hashes: RwLock<HashMap<String, String>>,
    pub stale_flag: AtomicBool,
    pub config: RwLock<ProjectConfig>,
    pub audit_sender: tokio::sync::mpsc::UnboundedSender<AuditEvent>,
    pub started_at: Instant,
}

impl EngineState {
    pub fn new(config: ProjectConfig) -> (Self, tokio::sync::mpsc::UnboundedReceiver<AuditEvent>) {
        let (audit_sender, audit_receiver) = tokio::sync::mpsc::unbounded_channel();
        (Self {
            graph: ArcSwap::new(Arc::new((StableGraph::<WikiPageMeta, EdgeType>::new(), HashMap::new()))),
            page_contents: DashMap::new(),
            section_corpus: ArcSwap::new(Arc::new(Vec::new())),
            bm25_index: ArcSwap::new(Arc::new(Bm25Index::new())),
            vector_registry: ArcSwap::new(Arc::new(HashMap::new())),
            source_registry: RwLock::new(HashMap::new()),
            content_hashes: RwLock::new(HashMap::new()),
            stale_flag: AtomicBool::new(true),
            config: RwLock::new(config),
            audit_sender,
            started_at: Instant::now(),
        }, audit_receiver)
    }
}

// ─── VppEngine ──────────────────────────────────────────────

pub struct VppEngine {
    pub state: Arc<EngineState>,
    pub _audit_handle: Option<tokio::task::JoinHandle<()>>,
}

impl VppEngine {
    pub fn new(config: ProjectConfig) -> Self {
        let (state, mut audit_receiver) = EngineState::new(config);
        let state = Arc::new(state);

        // Spawn audit log consumer
        let handle = tokio::spawn(async move {
            let log_path = std::path::Path::new(".wm").join("audit.jsonl");
            // Ensure directory exists
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }

            loop {
                match audit_receiver.recv().await {
                    Some(event) => {
                        // Skip help and initial tools
                        if event.tool_name == "help" || event.tool_name == "initial" {
                            continue;
                        }
                        if let Ok(line) = serde_json::to_string(&event) {
                            if let Ok(mut file) = std::fs::OpenOptions::new()
                                .create(true).append(true).open(&log_path)
                            {
                                use std::io::Write;
                                let _ = writeln!(file, "{}", line);
                                let _ = file.flush();
                            }
                        }
                    }
                    None => break, // Channel closed
                }
            }
        });

        Self { state, _audit_handle: Some(handle) }
    }

    pub fn mark_stale(&self) {
        self.state.stale_flag.store(true, std::sync::atomic::Ordering::Release);
    }
}
