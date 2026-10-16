use std::collections::BTreeMap;

use wm_engine::{
    canonical_questions, AnswerValue, DecisionRecord, MemoryLayer, PageType, RECORD_SCHEMA_VERSION,
};

use crate::engine::EngineState;
use crate::error::ToolError;
use wm_engine::parser;
use crate::search::{tokenize, Bm25Index};

const MEMORY_ID_PREFIX: &str = "wiki:memory:";
const LAYER_PROJECT: &str = "project";
const LAYER_GLOBAL: &str = "global";
const LAYER_SESSION: &str = "session";
const ERROR_GLOBAL_LAYER_UNIMPLEMENTED: &str =
    "the global memory layer is not implemented; use project or session (see wiki:specs:remove-global-memory-layer)";
const ERROR_UNKNOWN_LAYER: &str = "unknown memory layer; expected project or session";
const RECORD_BODY_RENDER_FAILURE: &str = "memory record body render failed";
const REASON_EXISTING_PAGE: &str = "memory page already exists";
const REASON_NEAR_DUPLICATE: &str = "near-duplicate memory found";
const ERROR_EMPTY_TITLE: &str = "memory title must not be empty";
const ERROR_EMPTY_CONTENT: &str = "memory content must not be empty";
const LOCK_PROJECT_ROOT: &str = "project_root";
const LOCK_CONFIG: &str = "config";
const ANSWER_LAYER: &str = "layer";
const ANSWER_STORE_OR_SKIP: &str = "store_or_skip";
const ANSWER_DEDUP_ACTION: &str = "dedup_action";
const ANSWER_CONFIDENCE: &str = "confidence";
const WIKI_REFERENCE_PREFIX: &str = "wiki:";
const WIKI_AT_PREFIX: &str = "@wiki/";
const PATH_SEPARATOR: char = '/';
const EXTENSION_SEPARATOR: char = '.';
const PERCENT_SUFFIX: char = '%';
const MEASUREMENT_UNITS: [&str; 6] = ["ms", "s", "mb", "kb", "gb", "x"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedupAction {
    New,
    Skip,
}

impl DedupAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Skip => "skip",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
}

impl Confidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkipReason {
    ExistingPage,
    NearDuplicate,
}

impl SkipReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExistingPage => REASON_EXISTING_PAGE,
            Self::NearDuplicate => REASON_NEAR_DUPLICATE,
        }
    }
}

#[derive(Debug, Default)]
pub struct GateEvidence {
    pub nearest_id: Option<String>,
    pub nearest_score: Option<f32>,
}

#[derive(Debug)]
pub struct MemoryWriteDecision {
    pub store: bool,
    pub dedup_action: DedupAction,
    pub layer: MemoryLayer,
    pub confidence: Confidence,
    pub evidence: GateEvidence,
    pub skip_reason: Option<SkipReason>,
}

pub fn evaluate_memory_write(
    engine: &EngineState,
    title: &str,
    content: &str,
    layer_hint: Option<&str>,
) -> Result<MemoryWriteDecision, ToolError> {
    if title.trim().is_empty() {
        return Err(ToolError::invalid_params(ERROR_EMPTY_TITLE));
    }
    if content.trim().is_empty() {
        return Err(ToolError::invalid_params(ERROR_EMPTY_CONTENT));
    }
    let root = engine
        .project_root
        .read()
        .map(|root| root.clone())
        .map_err(|_| ToolError::lock_poisoned(LOCK_PROJECT_ROOT))?;
    let layer = layer_from_hint(layer_hint)?;
    let slug = memory_slug(title);
    let target_id = format!("{MEMORY_ID_PREFIX}{slug}");
    let path_exists =
        crate::page::helpers::page_path_helper::resolve_id_to_path(&root, &target_id).is_ok();

    let index = engine.bm25_index.load();
    let nearest = nearest_memory_hit(&index, title, content);

    let threshold = engine
        .config
        .read()
        .map_err(|_| ToolError::lock_poisoned(LOCK_CONFIG))?
        .search
        .scoring
        .memory_dedup_threshold;
    let score_trips = nearest
        .as_ref()
        .map(|(_, score)| *score >= threshold)
        .unwrap_or(false);

    let store = !path_exists && !score_trips;
    let dedup_action = match store {
        true => DedupAction::New,
        false => DedupAction::Skip,
    };
    let skip_reason = match (store, path_exists) {
        (true, _) => None,
        (false, true) => Some(SkipReason::ExistingPage),
        (false, false) => Some(SkipReason::NearDuplicate),
    };

    Ok(MemoryWriteDecision {
        store,
        dedup_action,
        layer,
        confidence: classify_confidence(content),
        evidence: build_evidence(nearest, path_exists, &target_id),
        skip_reason,
    })
}

fn classify_confidence(content: &str) -> Confidence {
    if cites_concrete_anchor(content) {
        return Confidence::High;
    }
    unverifiable_provenance_reserved_for_low()
}

fn unverifiable_provenance_reserved_for_low() -> Confidence {
    Confidence::Medium
}

fn cites_concrete_anchor(content: &str) -> bool {
    cites_reference_id(content) || cites_file_path(content) || cites_measurement(content)
}

fn cites_reference_id(content: &str) -> bool {
    content.contains(WIKI_REFERENCE_PREFIX) || content.contains(WIKI_AT_PREFIX)
}

fn cites_file_path(content: &str) -> bool {
    content.split_whitespace().any(looks_like_path)
}

fn looks_like_path(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| !(c.is_alphanumeric() || c == PATH_SEPARATOR));
    let Some((prefix, last)) = trimmed.rsplit_once(PATH_SEPARATOR) else {
        return false;
    };
    if prefix.is_empty() || last.is_empty() {
        return false;
    }
    let Some((stem, extension)) = last.rsplit_once(EXTENSION_SEPARATOR) else {
        return false;
    };
    !stem.is_empty() && !extension.is_empty() && extension.chars().all(|c| c.is_ascii_alphabetic())
}

fn cites_measurement(content: &str) -> bool {
    content.split_whitespace().any(looks_like_measurement)
}

fn looks_like_measurement(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| {
        !(c.is_ascii_alphanumeric() || c == EXTENSION_SEPARATOR || c == PERCENT_SUFFIX)
    });
    if trimmed.is_empty() {
        return false;
    }
    if let Some(digits) = trimmed.strip_suffix(PERCENT_SUFFIX) {
        return is_decimal_number(digits);
    }
    let Some((digits, unit)) = split_number_unit(trimmed) else {
        return false;
    };
    is_decimal_number(digits)
        && MEASUREMENT_UNITS
            .iter()
            .any(|known| known.eq_ignore_ascii_case(unit))
}

fn split_number_unit(token: &str) -> Option<(&str, &str)> {
    let split = token.find(|c: char| c.is_ascii_alphabetic())?;
    let (digits, unit) = token.split_at(split);
    if digits.is_empty() || unit.is_empty() {
        return None;
    }
    Some((digits, unit))
}

fn is_decimal_number(text: &str) -> bool {
    let mut parts = text.split(EXTENSION_SEPARATOR);
    let whole = parts.next().unwrap_or_default();
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let Some(fraction) = parts.next() else {
        return true;
    };
    if fraction.is_empty() || !fraction.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    parts.next().is_none()
}

pub fn memory_slug(text: &str) -> String {
    let slug: String = text
        .to_lowercase()
        .chars()
        .map(|c| match c.is_alphanumeric() || c == '-' || c == '_' {
            true => c,
            false => '-',
        })
        .collect();

    let mut result = String::with_capacity(slug.len());
    let mut prev_hyphen = false;
    for c in slug.chars() {
        if c != '-' {
            result.push(c);
            prev_hyphen = false;
            continue;
        }
        if !prev_hyphen {
            result.push('-');
            prev_hyphen = true;
        }
    }
    result.trim_matches('-').to_owned()
}

fn layer_from_hint(hint: Option<&str>) -> Result<MemoryLayer, ToolError> {
    match hint {
        None | Some(LAYER_PROJECT) => Ok(MemoryLayer::Project),
        Some(LAYER_SESSION) => Ok(MemoryLayer::Session),
        Some(LAYER_GLOBAL) => Err(ToolError::invalid_params(ERROR_GLOBAL_LAYER_UNIMPLEMENTED)),
        Some(_) => Err(ToolError::invalid_params(ERROR_UNKNOWN_LAYER)),
    }
}

fn nearest_memory_hit(index: &Bm25Index, title: &str, content: &str) -> Option<(String, f64)> {
    let tokens = tokenize(&format!("{title}\n{content}"));
    if tokens.is_empty() {
        return None;
    }
    let mut best: Option<(String, f64)> = None;
    for doc in &index.docs {
        let page_id = doc.id.split_once('#').map(|(id, _)| id).unwrap_or(&doc.id);
        if !page_id.starts_with(MEMORY_ID_PREFIX) {
            continue;
        }
        let score = index.score_doc(doc, &tokens);
        if score <= 0.0 {
            continue;
        }
        if best.as_ref().map(|(_, top)| score > *top).unwrap_or(true) {
            best = Some((page_id.to_owned(), score));
        }
    }
    best
}

fn build_evidence(
    nearest: Option<(String, f64)>,
    path_exists: bool,
    target_id: &str,
) -> GateEvidence {
    if path_exists {
        return match nearest {
            Some((id, score)) if id == target_id => GateEvidence {
                nearest_id: Some(id),
                nearest_score: Some(score as f32),
            },
            _ => GateEvidence {
                nearest_id: Some(target_id.to_owned()),
                nearest_score: None,
            },
        };
    }
    match nearest {
        Some((id, score)) => GateEvidence {
            nearest_id: Some(id),
            nearest_score: Some(score as f32),
        },
        None => GateEvidence::default(),
    }
}

impl MemoryWriteDecision {
    pub fn answers_body(&self, content: &str) -> Result<String, ToolError> {
        let body = crate::page::build_record_body(&PageType::Memory, content).map_err(|error| {
            ToolError::internal(format!("{RECORD_BODY_RENDER_FAILURE}: {error}"))
        })?;
        let mut answers = BTreeMap::new();
        answers.insert(
            ANSWER_LAYER.to_owned(),
            AnswerValue::Label(self.layer.as_str().to_owned()),
        );
        answers.insert(
            ANSWER_STORE_OR_SKIP.to_owned(),
            AnswerValue::Bool(self.store),
        );
        answers.insert(
            ANSWER_DEDUP_ACTION.to_owned(),
            AnswerValue::Label(self.dedup_action.as_str().to_owned()),
        );
        answers.insert(
            ANSWER_CONFIDENCE.to_owned(),
            AnswerValue::Label(self.confidence.as_str().to_owned()),
        );
        let record = DecisionRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            state: content.to_owned(),
            questions: canonical_questions(&PageType::Memory),
            answers,
        };
        Ok(parser::write_record_answers(&body, &record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_slug_keeps_underscores_and_collapses_runs() {
        assert_eq!(memory_slug("my_task"), "my_task");
        assert_eq!(memory_slug("Auth & Security!"), "auth-security");
        assert_eq!(memory_slug("Hello  World"), "hello-world");
    }

    #[test]
    fn confidence_is_high_for_reference_ids_paths_and_measurements() {
        assert_eq!(classify_confidence("See wiki:specs:foo"), Confidence::High);
        assert_eq!(
            classify_confidence("Read @wiki/specs/foo"),
            Confidence::High
        );
        assert_eq!(
            classify_confidence("Edit apps/wm-core/src/lib.rs"),
            Confidence::High
        );
        assert_eq!(
            classify_confidence("Reduced latency by 40%"),
            Confidence::High
        );
        assert_eq!(
            classify_confidence("Warm start dropped to 300ms"),
            Confidence::High
        );
        assert_eq!(
            classify_confidence("Cut it to 1.5x throughput"),
            Confidence::High
        );
        assert_eq!(
            classify_confidence("Retried the request a second time"),
            Confidence::Medium
        );
    }

    #[test]
    fn confidence_is_medium_for_bare_numbers() {
        assert_eq!(
            classify_confidence("Retried the request 3 times"),
            Confidence::Medium
        );
        assert_eq!(classify_confidence("Released in 2026"), Confidence::Medium);
        assert_eq!(classify_confidence("Landed in v2"), Confidence::Medium);
        assert_eq!(
            classify_confidence("Run step5 before shipping"),
            Confidence::Medium
        );
    }

    #[test]
    fn confidence_is_medium_when_no_anchor_is_cited() {
        assert_eq!(
            classify_confidence("Prefer line-based YAML edits for durable memory."),
            Confidence::Medium
        );
    }

    #[test]
    fn unverifiable_provenance_predicate_is_the_reserved_low_seat() {
        assert_eq!(
            unverifiable_provenance_reserved_for_low(),
            Confidence::Medium
        );
    }

    #[test]
    fn dedup_action_and_skip_reason_stay_within_the_v1_vocabulary() {
        let emitted = [DedupAction::New, DedupAction::Skip];
        assert!(emitted
            .iter()
            .all(|action| matches!(action.as_str(), "new" | "skip")));
        assert_eq!(SkipReason::ExistingPage.as_str(), REASON_EXISTING_PAGE);
        assert_eq!(SkipReason::NearDuplicate.as_str(), REASON_NEAR_DUPLICATE);
    }

    #[test]
    fn layer_hint_maps_project_and_session_and_rejects_the_rest() {
        assert_eq!(
            layer_from_hint(None).expect("default layer"),
            MemoryLayer::Project
        );
        assert_eq!(
            layer_from_hint(Some(LAYER_PROJECT)).expect("project layer"),
            MemoryLayer::Project
        );
        assert_eq!(
            layer_from_hint(Some(LAYER_SESSION)).expect("session layer"),
            MemoryLayer::Session
        );
        assert_eq!(
            layer_from_hint(Some(LAYER_GLOBAL))
                .expect_err("global is not implemented")
                .code,
            "INVALID_PARAMS"
        );
        assert_eq!(
            layer_from_hint(Some("nonsense"))
                .expect_err("unknown layer")
                .code,
            "INVALID_PARAMS"
        );
    }
}
