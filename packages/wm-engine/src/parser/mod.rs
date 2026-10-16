use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde_yaml::{Mapping, Value};

use crate::helpers::yaml_helper::{yaml_quote, yaml_scalar};
use crate::models::edge_type_model::{EdgeType, GraphEdge};
use crate::models::page_data::{
    AcceptanceCriterion, DecisionData, FunctionalRequirement, GeneralGoal,
    NonFunctionalRequirement, PatternData, RuleCategory, RuleData, SpecData, TaskData,
};
use crate::models::page_type_model::PageType;
use crate::models::time_entry_model::TimeEntry;
use crate::models::WikiPageMeta;
use crate::record_state_text;
use crate::status::{Confidence, PageStatus, Priority};
use crate::SectionDoc;

pub mod record_writer;
pub use record_writer::*;

pub fn extract_frontmatter(content: &str) -> (Option<WikiPageMeta>, &str) {
    extract_frontmatter_from("<unknown source>", content)
}

const FRONTMATTER_OPEN: &str = "---";
const FRONTMATTER_CLOSE: &str = "\n---";
const FRONTMATTER_OPEN_LEN: usize = FRONTMATTER_OPEN.len();
const FRONTMATTER_CLOSE_LEN: usize = FRONTMATTER_CLOSE.len();

pub fn split_first_frontmatter(content: &str) -> Option<(&str, &str)> {
    let content = content.trim();
    let after_open = content.strip_prefix(FRONTMATTER_OPEN)?;
    let pos = after_open.find(FRONTMATTER_CLOSE)?;
    let close = FRONTMATTER_OPEN_LEN.saturating_add(pos);
    let yaml = content.get(FRONTMATTER_OPEN_LEN.saturating_add(1)..close.saturating_add(1))?;
    let body = content
        .get(close.saturating_add(FRONTMATTER_CLOSE_LEN)..)
        .unwrap_or_default()
        .trim();
    Some((yaml, body))
}

pub fn extract_raw_frontmatter(content: &str) -> (Option<String>, &str) {
    match split_first_frontmatter(content) {
        Some((yaml, body)) => (Some(yaml.to_string()), body),
        None => (None, content.trim()),
    }
}

pub fn extract_frontmatter_from<'a>(
    source: &str,
    content: &'a str,
) -> (Option<WikiPageMeta>, &'a str) {
    let Some((yaml, body)) = split_first_frontmatter(content) else {
        return (None, content.trim());
    };

    match build_page_meta(yaml) {
        Some(meta) => (Some(meta), body),
        None => {
            tracing::warn!(
                "Frontmatter parse error in {} — frontmatter was: {}",
                source,
                yaml.chars().take(160).collect::<String>()
            );
            (None, content.trim())
        }
    }
}

fn take(map: &mut Mapping, key: &str) -> Option<Value> {
    map.remove(Value::String(key.to_string()))
}

fn take_string(map: &mut Mapping, key: &str) -> Result<Option<String>, ()> {
    let Some(value) = take(map, key) else {
        return Ok(None);
    };
    serde_yaml::from_value::<String>(value)
        .map(Some)
        .map_err(|error| {
            tracing::warn!("Frontmatter field '{}' failed to parse: {}", key, error);
        })
}

fn take_value<T: DeserializeOwned>(map: &mut Mapping, key: &str) -> Result<Option<T>, ()> {
    let Some(value) = take(map, key) else {
        return Ok(None);
    };
    serde_yaml::from_value(value).map(Some).map_err(|error| {
        tracing::warn!("Frontmatter field '{}' failed to parse: {}", key, error);
    })
}

fn relation_pair(value: &Value) -> Result<(EdgeType, String, String), ()> {
    let Some(mapping) = value.as_mapping() else {
        tracing::warn!("Frontmatter relates_to entry is not a mapping");
        return Err(());
    };
    let edge_type = mapping
        .get(Value::String("type".to_string()))
        .and_then(Value::as_str);
    let target = mapping
        .get(Value::String("target".to_string()))
        .and_then(Value::as_str);
    match (edge_type, target) {
        (Some(edge_type), Some(target)) => Ok((
            EdgeType::from_str_flexible(edge_type),
            edge_type.to_string(),
            target.to_string(),
        )),
        _ => {
            tracing::warn!("Frontmatter relates_to entry is missing 'type' or 'target'");
            Err(())
        }
    }
}

fn parse_relation_pairs(value: &Value) -> Result<Vec<(EdgeType, String, String)>, ()> {
    let Some(sequence) = value.as_sequence() else {
        tracing::warn!("Frontmatter field 'relates_to' is not a sequence");
        return Err(());
    };
    sequence.iter().map(relation_pair).collect()
}

fn acceptance_criterion(value: &Value) -> Result<AcceptanceCriterion, ()> {
    if let Some(text) = value.as_str() {
        return Ok(AcceptanceCriterion {
            text: text.to_string(),
            checked: false,
        });
    }
    let Some(mapping) = value.as_mapping() else {
        tracing::warn!("Frontmatter acceptance_criteria entry is not a string or mapping");
        return Err(());
    };
    let Some(text) = mapping
        .get(Value::String("text".to_string()))
        .and_then(Value::as_str)
    else {
        tracing::warn!("Frontmatter acceptance_criteria entry is missing 'text'");
        return Err(());
    };
    let checked = mapping
        .get(Value::String("checked".to_string()))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(AcceptanceCriterion {
        text: text.to_string(),
        checked,
    })
}

fn parse_acceptance_criteria(value: &Value) -> Result<Vec<AcceptanceCriterion>, ()> {
    let Some(sequence) = value.as_sequence() else {
        tracing::warn!("Frontmatter field 'acceptance_criteria' is not a sequence");
        return Err(());
    };
    sequence.iter().map(acceptance_criterion).collect()
}

fn build_page_meta(yaml: &str) -> Option<WikiPageMeta> {
    let mut map = match serde_yaml::from_str::<Value>(yaml) {
        Ok(Value::Mapping(map)) => map,
        Ok(other) => {
            tracing::warn!("Frontmatter is not a mapping: {:?}", other);
            return None;
        }
        Err(error) => {
            tracing::warn!("Frontmatter YAML parse failed: {}", error);
            return None;
        }
    };

    let title = take_string(&mut map, "title").ok()?.unwrap_or_default();
    take(&mut map, "id");
    let id = frontmatter_id_from_raw(yaml).unwrap_or_default();
    let raw_type = take_string(&mut map, "type").ok()?;
    let page_type = raw_type
        .as_deref()
        .and_then(|name| PageType::from_type_name(&name.to_lowercase()))
        .unwrap_or(PageType::Concept);
    let raw_status = take_string(&mut map, "status").ok()?;
    let status = raw_status
        .as_deref()
        .map(parse_page_status)
        .unwrap_or(PageStatus::Draft);
    let tags = take_value::<Vec<String>>(&mut map, "tags")
        .ok()?
        .unwrap_or_default();
    let aliases = take_value::<Vec<String>>(&mut map, "aliases")
        .ok()?
        .unwrap_or_default();
    let sources = take_value::<Vec<String>>(&mut map, "sources")
        .ok()?
        .unwrap_or_default();
    let raw_priority = take_string(&mut map, "priority").ok()?;
    let priority = raw_priority.as_deref().and_then(parse_priority);
    let raw_confidence = take_string(&mut map, "confidence").ok()?;
    let confidence = raw_confidence
        .as_deref()
        .and_then(|value| serde_yaml::from_str::<Confidence>(value).ok());
    let assignee = take_string(&mut map, "assignee").ok()?;
    let superseded_by = take_string(&mut map, "superseded_by").ok()?;
    let version = take_string(&mut map, "version").ok()?;
    let parent = take_string(&mut map, "parent").ok()?;
    let source_url = take_string(&mut map, "source_url").ok()?;
    let time_started = take_string(&mut map, "time_started").ok()?;
    let order = take_value::<i32>(&mut map, "order").ok()?;
    let relates_to = take(&mut map, "relates_to")
        .map(|value| parse_relation_pairs(&value))
        .transpose()
        .ok()?
        .unwrap_or_default();
    let (relates_to, raw_relates_to) = relates_to
        .into_iter()
        .map(|(edge_type, raw_edge_type, target)| {
            ((edge_type, target.clone()), (raw_edge_type, target))
        })
        .unzip();

    let acceptance_criteria = take(&mut map, "acceptance_criteria")
        .map(|value| parse_acceptance_criteria(&value))
        .transpose()
        .ok()?
        .unwrap_or_default();
    let estimate = take_value::<u32>(&mut map, "estimate").ok()?;
    let prerequisites = take_value::<Vec<String>>(&mut map, "prerequisites")
        .ok()?
        .unwrap_or_default();
    let difficulty = take_string(&mut map, "difficulty").ok()?;
    let time_spent = take_string(&mut map, "time_spent").ok()?;
    let time_entries = take_value::<Vec<TimeEntry>>(&mut map, "time_entries")
        .ok()?
        .unwrap_or_default();
    let implementation_plan = take_string(&mut map, "implementation_plan").ok()?;
    let implementation_notes = take_string(&mut map, "implementation_notes").ok()?;
    let has_task = !acceptance_criteria.is_empty()
        || estimate.is_some()
        || !prerequisites.is_empty()
        || difficulty.is_some()
        || time_spent.is_some()
        || !time_entries.is_empty()
        || implementation_plan.is_some()
        || implementation_notes.is_some();
    let task_data = has_task.then_some(TaskData {
        acceptance_criteria,
        estimate,
        prerequisites,
        difficulty,
        time_spent,
        time_entries,
        implementation_plan,
        implementation_notes,
    });

    let functional_requirements =
        take_value::<Vec<FunctionalRequirement>>(&mut map, "functional_requirements")
            .ok()?
            .unwrap_or_default();
    let non_functional_requirements =
        take_value::<Vec<NonFunctionalRequirement>>(&mut map, "non_functional_requirements")
            .ok()?
            .unwrap_or_default();
    let general_goals = take_value::<Vec<GeneralGoal>>(&mut map, "general_goals")
        .ok()?
        .unwrap_or_default();
    let stakeholders = take_value::<Vec<String>>(&mut map, "stakeholders")
        .ok()?
        .unwrap_or_default();
    let has_spec = !functional_requirements.is_empty()
        || !non_functional_requirements.is_empty()
        || !general_goals.is_empty()
        || !stakeholders.is_empty();
    let spec_data = has_spec.then_some(SpecData {
        functional_requirements,
        non_functional_requirements,
        general_goals,
        stakeholders,
    });

    let decision_data = take_value::<DecisionData>(&mut map, "decision").ok()?;
    let pattern_data = take_value::<PatternData>(&mut map, "pattern").ok()?;
    let category = take_value::<RuleCategory>(&mut map, "category").ok()?;
    let raw_rationale = take_string(&mut map, "rationale").ok()?;
    let raw_example = take_string(&mut map, "example").ok()?;
    let raw_anti_pattern = take_string(&mut map, "anti_pattern").ok()?;
    let rule_data = category.map(|category| RuleData {
        category,
        rationale: raw_rationale.clone().unwrap_or_default(),
        example: raw_example.clone(),
        anti_pattern: raw_anti_pattern.clone(),
    });

    let unknown = map
        .into_iter()
        .filter_map(|(key, value)| key.as_str().map(|key| (key.to_string(), value)))
        .collect::<BTreeMap<String, Value>>();

    Some(WikiPageMeta {
        id,
        title,
        tags,
        status,
        published: false,
        priority,
        confidence,
        assignee,
        aliases,
        superseded_by,
        version,
        sources,
        parent,
        relates_to,
        path: PathBuf::new(),
        created_at: String::new(),
        updated_at: String::new(),
        page_type,
        order,
        task_data,
        spec_data,
        decision_data,
        pattern_data,
        memory_data: None,
        rule_data,
        raw_type,
        raw_status,
        raw_priority,
        raw_confidence,
        raw_relates_to,
        raw_rationale,
        raw_example,
        raw_anti_pattern,
        time_started,
        source_url,
        unknown,
    })
}

pub fn split_sections(raw: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut in_code_fence = false;
    let mut current_header = "Overview".into();
    let mut current_body = String::new();

    for line in raw.lines() {
        if line.trim().starts_with("```") {
            in_code_fence = !in_code_fence;
        }

        if line.starts_with("## ") && !in_code_fence {
            if !current_body.trim().is_empty() {
                sections.push((current_header, current_body.trim().to_string()));
            }
            current_header = line.trim_start_matches("## ").to_string();
            current_body.clear();
            continue;
        }
        current_body.push_str(line);
        current_body.push('\n');
    }
    if !current_body.trim().is_empty() {
        sections.push((current_header, current_body.trim().to_string()));
    }
    sections
}

pub fn readable_body(body: &str) -> String {
    record_state_text(body).unwrap_or_else(|| body.trim().to_owned())
}

pub fn parse_page_status(s: &str) -> PageStatus {
    match s.to_lowercase().as_str() {
        "todo" => PageStatus::Todo,
        "in-progress" | "wip" => PageStatus::InProgress,
        "in-review" | "reviewing" | "in_review" => PageStatus::InReview,
        "done" | "complete" => PageStatus::Done,
        "blocked" => PageStatus::Blocked,
        "cancelled" | "canceled" => PageStatus::Cancelled,
        "draft" => PageStatus::Draft,
        "reviewed" | "review" => PageStatus::Reviewed,
        "superseded" => PageStatus::Superseded,
        "approved" | "accepted" => PageStatus::Approved,
        "active" => PageStatus::Active,
        "stale" => PageStatus::Stale,
        "rejected" => PageStatus::Rejected,
        "archived" => PageStatus::Archived,
        other => {
            tracing::warn!(
                "Unknown page status string: '{}', defaulting to Draft",
                other
            );
            PageStatus::Draft
        }
    }
}

pub fn parse_priority(s: &str) -> Option<Priority> {
    match s.to_lowercase().as_str() {
        "low" => Some(Priority::Low),
        "medium" | "med" => Some(Priority::Medium),
        "high" => Some(Priority::High),
        "urgent" | "critical" => Some(Priority::Urgent),
        _ => None,
    }
}

pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn extract_wikilinks(text: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"\[\[([^\]]+?)(?:\|[^\]]+)?\]\]")
            .expect("hardcoded wikilink pattern should be valid")
    });
    re.captures_iter(text)
        .map(|cap| cap[1].trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn extract_inline_tags(text: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(r"(?:^|\s)#([a-zA-Z][\w-]*)")
            .expect("hardcoded inline-tag pattern should be valid")
    });

    let mut tags = Vec::new();
    let mut in_code_fence = false;

    for line in text.lines() {
        if line.trim().starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        if line.trim_start().starts_with("## ") {
            continue;
        }

        for cap in re.captures_iter(line) {
            let tag = cap[1].to_string();
            if !tag.is_empty() && tag.len() > 1 {
                tags.push(tag);
            }
        }
    }

    tags
}

pub fn resolve_link_target_candidates(
    target: &str,
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
) -> Vec<String> {
    let target_lower = target.to_lowercase();
    let target_norm = target_lower.replace('-', " ");

    let mut candidates = Vec::new();
    for idx in graph.node_indices() {
        let meta = &graph[idx];

        if meta.id.to_lowercase() == target_lower {
            candidates.push(meta.id.clone());
            continue;
        }

        if let Some(last) = meta.id.split(':').next_back() {
            if last.to_lowercase() == target_lower {
                candidates.push(meta.id.clone());
                continue;
            }
        }

        let title_norm = meta.title.to_lowercase().replace('-', " ");
        if title_norm == target_norm {
            candidates.push(meta.id.clone());
            continue;
        }

        if title_norm.contains(&target_norm) || target_norm.contains(&title_norm) {
            candidates.push(meta.id.clone());
            continue;
        }

        if meta
            .aliases
            .iter()
            .any(|a| a.to_lowercase().replace('-', " ") == target_norm)
        {
            candidates.push(meta.id.clone());
        }
    }

    candidates
}

pub fn resolve_link_target(
    target: &str,
    graph: &petgraph::stable_graph::StableGraph<WikiPageMeta, GraphEdge>,
) -> Option<String> {
    resolve_link_target_candidates(target, graph)
        .into_iter()
        .next()
}

pub fn path_to_id(rel_path: &str) -> String {
    let after_wiki = rel_path.split(".wm/wiki/").last().unwrap_or(rel_path);
    let cleaned = after_wiki
        .trim_start_matches("./")
        .trim_start_matches(".wm/")
        .trim_start_matches("wiki/")
        .trim_start_matches("wm/")
        .trim_start_matches('/');
    let base = cleaned
        .strip_suffix(".md")
        .unwrap_or(cleaned)
        .replace('/', ":");
    if !base.starts_with("wiki:") && !base.is_empty() {
        return format!("wiki:{}", base);
    }
    base
}

pub fn parse_wiki_page(file_path: &Path, content: &str) -> WikiPageMeta {
    let (extracted, body) = extract_frontmatter_from(&file_path.to_string_lossy(), content);
    let mut meta = extracted.unwrap_or_default();

    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    meta.id = path_to_id(&rel_path);
    meta.path = file_path.to_path_buf();
    meta.published = false;
    meta.created_at = String::new();
    meta.updated_at = String::new();

    if meta.title.trim().is_empty() {
        meta.title = file_path
            .file_stem()
            .map(|s| s.to_string_lossy().replace('-', " "))
            .unwrap_or_default();
    }

    let searchable = record_state_text(body).unwrap_or_else(|| body.to_owned());

    for tag in extract_inline_tags(&searchable) {
        if !meta.tags.contains(&tag) {
            meta.tags.push(tag);
        }
    }

    for link in extract_wikilinks(&searchable) {
        let entry = (EdgeType::RelatesTo, link);
        if !meta.relates_to.contains(&entry) {
            meta.relates_to.push(entry);
        }
    }

    meta
}

pub fn parse_sections(file_path: &Path, content: &str) -> Vec<SectionDoc> {
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let page_id = path_to_id(&rel_path);

    let (meta, body) = extract_frontmatter_from(&rel_path, content);
    let searchable = record_state_text(body).unwrap_or_else(|| body.to_owned());
    let title = meta
        .as_ref()
        .map(|m| m.title.clone())
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| {
            file_path
                .file_stem()
                .map(|s| s.to_string_lossy().replace('-', " "))
                .unwrap_or_default()
        });
    let mut tags: Vec<String> = meta.as_ref().map(|m| m.tags.clone()).unwrap_or_default();
    for tag in extract_inline_tags(&searchable) {
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    let sections = split_sections(&searchable);

    sections
        .into_iter()
        .map(|(header, body)| {
            let section_id = format!("{}#{}", page_id, header.to_lowercase().replace(' ', "-"));
            SectionDoc {
                section_id,
                page_id: page_id.clone(),
                header,
                body,
                title: title.clone(),
                tags: tags.clone(),
            }
        })
        .collect()
}

pub fn frontmatter_to_yaml(meta: &WikiPageMeta) -> String {
    let mut yaml = String::new();

    append_scalar_fields(&mut yaml, meta);
    append_time_entries(&mut yaml, meta);
    append_requirements(&mut yaml, meta);
    append_decision(&mut yaml, meta);
    append_pattern(&mut yaml, meta);
    append_relates_to(&mut yaml, meta);
    append_acceptance_criteria(&mut yaml, meta);
    append_rule_fields(&mut yaml, meta);
    append_unknown_fields(&mut yaml, meta);

    yaml
}

fn append_scalar_fields(yaml: &mut String, meta: &WikiPageMeta) {
    if !meta.title.trim().is_empty() {
        yaml.push_str(&format!("title: {}\n", yaml_scalar(&meta.title)));
    }
    if !meta.id.trim().is_empty() {
        yaml.push_str(&format!("id: {}\n", yaml_quote(&meta.id)));
    }
    if let Some(ref raw_type) = meta.raw_type {
        yaml.push_str(&format!("type: {}\n", raw_type));
    }
    if !meta.tags.is_empty() {
        yaml.push_str(&format!("tags: [{}]\n", meta.tags.join(", ")));
    }
    if let Some(ref raw_status) = meta.raw_status {
        yaml.push_str(&format!("status: {}\n", raw_status));
    }
    if let Some(ref raw_priority) = meta.raw_priority {
        yaml.push_str(&format!("priority: {}\n", raw_priority));
    }
    if let Some(ref raw_confidence) = meta.raw_confidence {
        yaml.push_str(&format!("confidence: {}\n", raw_confidence));
    }
    if let Some(ref assignee) = meta.assignee {
        yaml.push_str(&format!("assignee: {}\n", assignee));
    }
    if !meta.aliases.is_empty() {
        yaml.push_str(&format!("aliases: [{}]\n", meta.aliases.join(", ")));
    }
    if !meta.sources.is_empty() {
        yaml.push_str(&format!("sources: [{}]\n", meta.sources.join(", ")));
    }
    if let Some(ref superseded_by) = meta.superseded_by {
        yaml.push_str(&format!("superseded_by: {}\n", superseded_by));
    }
    if let Some(ref version) = meta.version {
        yaml.push_str(&format!("version: {}\n", version));
    }
    if let Some(task) = meta.task_data.as_ref() {
        if let Some(ref estimate) = task.estimate {
            yaml.push_str(&format!("estimate: {}\n", estimate));
        }
        if !task.prerequisites.is_empty() {
            yaml.push_str(&format!(
                "prerequisites: [{}]\n",
                task.prerequisites.join(", ")
            ));
        }
        if let Some(ref difficulty) = task.difficulty {
            yaml.push_str(&format!("difficulty: {}\n", difficulty));
        }
    }
    if let Some(ref source_url) = meta.source_url {
        yaml.push_str(&format!("source_url: {}\n", source_url));
    }
    if let Some(spec) = meta.spec_data.as_ref() {
        if !spec.stakeholders.is_empty() {
            yaml.push_str(&format!(
                "stakeholders: [{}]\n",
                spec.stakeholders.join(", ")
            ));
        }
    }
    if let Some(ref time_started) = meta.time_started {
        yaml.push_str(&format!("time_started: {}\n", time_started));
    }
    if let Some(task) = meta.task_data.as_ref() {
        if let Some(ref time_spent) = task.time_spent {
            yaml.push_str(&format!("time_spent: {}\n", time_spent));
        }
    }
    if let Some(ref order) = meta.order {
        yaml.push_str(&format!("order: {}\n", order));
    }
    if let Some(task) = meta.task_data.as_ref() {
        if let Some(ref plan) = task.implementation_plan {
            yaml.push_str(&format!("implementation_plan: \"{}\"\n", plan));
        }
        if let Some(ref notes) = task.implementation_notes {
            yaml.push_str(&format!("implementation_notes: \"{}\"\n", notes));
        }
    }
}

fn append_time_entries(yaml: &mut String, meta: &WikiPageMeta) {
    let Some(entries) = meta.task_data.as_ref().map(|task| &task.time_entries) else {
        return;
    };
    if entries.is_empty() {
        return;
    }
    yaml.push_str("time_entries:\n");
    for entry in entries {
        yaml.push_str(&format!("  - started_at: \"{}\"\n", entry.started_at));
        if let Some(ref ended) = entry.ended_at {
            yaml.push_str(&format!("    ended_at: \"{}\"\n", ended));
        }
        if let Some(dur) = entry.duration_s {
            yaml.push_str(&format!("    duration_s: {}\n", dur));
        }
        if let Some(ref note) = entry.note {
            yaml.push_str(&format!("    note: \"{}\"\n", note));
        }
    }
}

fn append_requirements(yaml: &mut String, meta: &WikiPageMeta) {
    let Some(spec) = meta.spec_data.as_ref() else {
        return;
    };
    if !spec.functional_requirements.is_empty() {
        yaml.push_str("functional_requirements:\n");
        for fr in &spec.functional_requirements {
            yaml.push_str(&format!(
                "  - {{id: {}, description: \"{}\"}}\n",
                fr.id, fr.description
            ));
        }
    }
    if !spec.non_functional_requirements.is_empty() {
        yaml.push_str("non_functional_requirements:\n");
        for nfr in &spec.non_functional_requirements {
            yaml.push_str(&format!(
                "  - {{id: {}, description: \"{}\"}}\n",
                nfr.id, nfr.description
            ));
        }
    }
    if !spec.general_goals.is_empty() {
        yaml.push_str("general_goals:\n");
        for goal in &spec.general_goals {
            yaml.push_str(&format!("  - {{description: \"{}\"}}\n", goal.description));
        }
    }
}

fn append_decision(yaml: &mut String, meta: &WikiPageMeta) {
    let Some(decision) = meta.decision_data.as_ref() else {
        return;
    };
    yaml.push_str("decision:\n");
    yaml.push_str(&format!("  context: \"{}\"\n", decision.context));
    if !decision.options.is_empty() {
        yaml.push_str(&format!("  options: [{}]\n", decision.options.join(", ")));
    }
    yaml.push_str(&format!("  rationale: \"{}\"\n", decision.rationale));
    yaml.push_str(&format!("  outcome: \"{}\"\n", decision.outcome));
    if let Some(ref consequences) = decision.consequences {
        yaml.push_str(&format!("  consequences: \"{}\"\n", consequences));
    }
}

fn append_pattern(yaml: &mut String, meta: &WikiPageMeta) {
    let Some(pattern) = meta.pattern_data.as_ref() else {
        return;
    };
    yaml.push_str("pattern:\n");
    yaml.push_str(&format!("  when_to_use: \"{}\"\n", pattern.when_to_use));
    yaml.push_str(&format!("  example: \"{}\"\n", pattern.example));
}

fn append_relates_to(yaml: &mut String, meta: &WikiPageMeta) {
    if meta.relates_to.is_empty() {
        return;
    }
    yaml.push_str("relates_to:\n");
    if meta.raw_relates_to.len() == meta.relates_to.len() {
        for (edge_type, target) in &meta.raw_relates_to {
            yaml.push_str(&format!(
                "  - {{type: {}, target: {}}}\n",
                edge_type, target
            ));
        }
        return;
    }
    for (edge_type, target) in &meta.relates_to {
        yaml.push_str(&format!(
            "  - {{type: {}, target: {}}}\n",
            edge_type.as_yaml_str(),
            target
        ));
    }
}

fn append_acceptance_criteria(yaml: &mut String, meta: &WikiPageMeta) {
    let Some(criteria) = meta
        .task_data
        .as_ref()
        .map(|task| &task.acceptance_criteria)
    else {
        return;
    };
    if criteria.is_empty() {
        return;
    }
    yaml.push_str("acceptance_criteria:\n");
    for criterion in criteria {
        yaml.push_str(&format!(
            "  - {{text: \"{}\", checked: {}}}\n",
            criterion.text, criterion.checked
        ));
    }
}

fn append_rule_fields(yaml: &mut String, meta: &WikiPageMeta) {
    if let Some(rule) = meta.rule_data.as_ref() {
        yaml.push_str(&format!("category: {:?}\n", rule.category));
    }
    if let Some(ref rationale) = meta.raw_rationale {
        yaml.push_str(&format!("rationale: \"{}\"\n", rationale));
    }
    if let Some(ref example) = meta.raw_example {
        yaml.push_str(&format!("example: \"{}\"\n", example));
    }
    if let Some(ref anti_pattern) = meta.raw_anti_pattern {
        yaml.push_str(&format!("anti_pattern: \"{}\"\n", anti_pattern));
    }
}

fn append_unknown_fields(yaml: &mut String, meta: &WikiPageMeta) {
    for (key, value) in &meta.unknown {
        if key == "id" {
            continue;
        }
        let rendered = serde_yaml::to_string(value).unwrap_or_default();
        let rendered = rendered.trim_end();
        if rendered.contains('\n') {
            yaml.push_str(&format!("{}:\n", key));
            for line in rendered.lines() {
                yaml.push_str(&format!("  {}\n", line));
            }
            continue;
        }
        yaml.push_str(&format!("{}: {}\n", key, rendered));
    }
}

pub fn count_frontmatter_blocks(content: &str) -> usize {
    let mut rest = content.trim_start();
    let mut count = 0usize;
    while rest.starts_with("---") {
        let Some(pos) = rest[3..].find("\n---") else {
            break;
        };
        let end = 3usize.wrapping_add(pos);
        rest = rest[end.wrapping_add(4)..].trim_start();
        count = count.wrapping_add(1);
    }
    count
}

pub fn extract_raw_first_frontmatter(content: &str) -> Option<&str> {
    let yaml = split_first_frontmatter(content)?.0;
    Some(yaml.strip_suffix('\n').unwrap_or(yaml))
}

pub fn frontmatter_id_from_raw(raw_fm: &str) -> Option<String> {
    frontmatter_id_raw_from_raw(raw_fm).map(|v| {
        let v = v.trim();
        v.trim_matches('"').trim_matches('\'').to_string()
    })
}

pub fn frontmatter_id_raw_from_raw(raw_fm: &str) -> Option<String> {
    for line in raw_fm.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.trim_end() == "id" {
                return Some(value.trim().to_string());
            }
        }
    }
    None
}

pub fn looks_like_scientific_notation_id(value: &str) -> bool {
    if value.is_empty() || value.contains('"') || value.contains('\'') {
        return false;
    }
    let mut has_e = false;
    for (i, c) in value.char_indices() {
        if c == 'e' || c == 'E' {
            if has_e || i == 0 || i == value.len() - 1 {
                return false;
            }
            has_e = true;
            continue;
        }
        if !c.is_ascii_digit() {
            return false;
        }
    }
    has_e
}

pub struct FrontmatterHealth {
    pub scientific_notation_id: Option<String>,
    pub duplicate_blocks: bool,
    pub id_mismatch: Option<String>,
}

pub fn inspect_frontmatter_health(content: &str, filename_stem: &str) -> FrontmatterHealth {
    let blocks = count_frontmatter_blocks(content);
    let raw = extract_raw_first_frontmatter(content);
    let frontmatter_id = raw.and_then(frontmatter_id_from_raw);
    let raw_id_line = raw.and_then(frontmatter_id_raw_from_raw);

    let scientific_notation_id = raw_id_line
        .as_deref()
        .filter(|v| looks_like_scientific_notation_id(v))
        .map(|v| v.trim_matches('"').trim_matches('\'').to_string());

    let id_mismatch = frontmatter_id.as_deref().and_then(|fid| {
        let normalized = fid
            .strip_prefix("wiki:tasks:")
            .or_else(|| fid.strip_prefix("wiki:"))
            .unwrap_or(fid);
        if filename_stem.is_empty() || normalized == filename_stem {
            return None;
        }
        Some(format!(
            "Frontmatter id '{}' does not match filename stem '{}'",
            fid, filename_stem
        ))
    });

    FrontmatterHealth {
        scientific_notation_id,
        duplicate_blocks: blocks >= 2,
        id_mismatch,
    }
}

pub fn parse_edge_type_flexible(s: &str) -> crate::EdgeType {
    crate::EdgeType::from_str_flexible(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_fence_ignores_headers() {
        let md = "\
## Real header

Some text

```
## This is inside a code fence
## Should not be parsed as a header
```

## Another real header

More text";

        let sections = split_sections(md);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].0, "Real header");
        assert_eq!(sections[1].0, "Another real header");
    }

    #[test]
    fn test_frontmatter_extraction() {
        let md = "\
---
title: Authentication Architecture
type: concept
tags: [auth, security]
status: reviewed
relates_to:
  - {type: extends, target: wiki:concepts:base-auth}
  - {type: implements, target: wiki:specs:auth-v2}
---

# Auth Architecture

Content here.";

        let (fm, body) = extract_frontmatter(md);
        assert!(fm.is_some());
        let fm = fm.unwrap();
        assert_eq!(fm.title, "Authentication Architecture");
        assert_eq!(fm.relates_to.len(), 2);
        assert_eq!(fm.relates_to[0].0, EdgeType::Extends);
        assert_eq!(fm.relates_to[0].1, "wiki:concepts:base-auth");
        assert!(body.contains("Content here"));
    }

    #[test]
    fn test_path_inferred_id() {
        let path = Path::new("wiki/concepts/auth.md");
        let meta = parse_wiki_page(path, "# Hello\n\nWorld");
        assert_eq!(meta.id, "wiki:concepts:auth");
    }

    #[test]
    fn test_content_hash() {
        let h1 = content_hash("hello world");
        let h2 = content_hash("hello world");
        let h3 = content_hash("hello world!");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_extract_wikilinks_basic() {
        let md = "This links to [[auth-service]] and [[oauth2|OAuth 2.0 Flow]].";
        let links = extract_wikilinks(md);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"auth-service".into()));
        assert!(links.contains(&"oauth2".into()));
    }

    #[test]
    fn test_extract_wikilinks_no_links() {
        let md = "This text has no wiki links.";
        let links = extract_wikilinks(md);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_wikilinks_code_fence() {
        let md = "```\n[[not-a-link]]\n```\n\n[[real-link]]";
        let links = extract_wikilinks(md);
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn test_extract_inline_tags_basic() {
        let md = "This is about #auth and #security features.";
        let tags = extract_inline_tags(md);
        assert!(tags.contains(&"auth".into()));
        assert!(tags.contains(&"security".into()));
    }

    #[test]
    fn test_extract_inline_tags_ignores_headings() {
        let md = "\
## Not a tag

#also-not-a-tag

This is #auth";
        let tags = extract_inline_tags(md);
        assert!(!tags.contains(&"Not".into()));
        assert!(tags.contains(&"auth".into()));
    }

    #[test]
    fn test_frontmatter_roundtrip() {
        let content = "---\ntitle: Test\ntype: task\nstatus: todo\n---\n\nBody here.";
        let (fm, body) = extract_frontmatter(content);
        assert!(fm.is_some(), "frontmatter should be parsed");
        assert_eq!(body.trim(), "Body here.");
        let fm = fm.unwrap();
        assert_eq!(fm.title, "Test");
    }

    #[test]
    fn test_split_first_frontmatter_preserves_raw_slices() {
        let content = "---\ntitle: One\ntype: concept\n---\n\n## Body\ntext\n";
        let (raw, body) = extract_raw_frontmatter(content);
        assert_eq!(raw.as_deref(), Some("title: One\ntype: concept\n"));
        assert_eq!(body, "## Body\ntext");
        assert_eq!(
            extract_raw_first_frontmatter(content),
            Some("title: One\ntype: concept")
        );
    }

    #[test]
    fn test_split_first_frontmatter_empty_block() {
        let content = "---\n---\n\nBody";
        let (raw, body) = extract_raw_frontmatter(content);
        assert_eq!(raw.as_deref(), Some(""));
        assert_eq!(body, "Body");
        assert_eq!(extract_raw_first_frontmatter(content), Some(""));
    }

    #[test]
    fn test_split_first_frontmatter_absent() {
        let content = "# Heading\n\nno frontmatter\n";
        assert_eq!(
            extract_raw_frontmatter(content),
            (None, "# Heading\n\nno frontmatter")
        );
        assert_eq!(extract_raw_first_frontmatter(content), None);
    }

    #[test]
    fn test_parse_wiki_page_with_obsidian_elements() {
        let md = "\
---
title: Auth Module
type: concept
tags: [backend]
---

# Auth Module

This module implements [[session-management]] for #security.

See also [[permissions|Permissions List]].";
        let path = Path::new("wiki/concepts/auth-module.md");
        let meta = parse_wiki_page(path, md);
        assert_eq!(meta.id, "wiki:concepts:auth-module");
        assert!(meta.tags.contains(&"backend".into()));
        assert!(meta.tags.contains(&"security".into()));
        assert!(meta
            .relates_to
            .iter()
            .any(|(_, target)| target == "session-management"));
        assert!(meta
            .relates_to
            .iter()
            .any(|(_, target)| target == "permissions"));
    }

    #[test]
    fn test_parse_wiki_page_extracts_tags_and_wikilinks_from_record_state() {
        let md = "\
---
title: Record
type: concept
---

schema_version: 1
state: |-
  ## Context

  See [[session-management]] for #security.
questions:
  - id: kind
    type: choice
    instructions: What kind of concept document is this?
    options: [concept, failure-analysis]
answers:
  kind: concept
";
        let path = Path::new("wiki/concepts/record.md");
        let meta = parse_wiki_page(path, md);
        assert!(
            meta.tags.contains(&"security".to_owned()),
            "inline tag inside state must be extracted"
        );
        assert!(
            meta.relates_to
                .iter()
                .any(|(_, target)| target == "session-management"),
            "wikilink inside state must be extracted"
        );
    }

    #[test]
    fn test_parse_edge_type_flexible_all() {
        assert_eq!(parse_edge_type_flexible("related"), EdgeType::RelatesTo);
        assert_eq!(parse_edge_type_flexible("relates-to"), EdgeType::RelatesTo);
        assert_eq!(parse_edge_type_flexible("depends-on"), EdgeType::DependsOn);
        assert_eq!(parse_edge_type_flexible("example-of"), EdgeType::ExampleOf);
        assert_eq!(parse_edge_type_flexible("part-of"), EdgeType::PartOf);
        assert_eq!(
            parse_edge_type_flexible("supports"),
            EdgeType::Custom("supports".into())
        );
        assert_eq!(
            parse_edge_type_flexible("contradicts"),
            EdgeType::Custom("contradicts".into())
        );
        assert_eq!(
            parse_edge_type_flexible("similar_to"),
            EdgeType::Custom("similar_to".into())
        );
        assert_eq!(
            parse_edge_type_flexible("custom-type"),
            EdgeType::Custom("custom-type".into())
        );
    }

    #[test]
    fn test_path_to_id_format() {
        let id = path_to_id("tasks/my-task.md");
        assert_eq!(
            id, "wiki:tasks:my-task",
            "expected wiki:tasks:my-task, got {}",
            id
        );
    }

    #[test]
    fn test_looks_like_scientific_notation_id() {
        assert!(looks_like_scientific_notation_id("652e07"));
        assert!(looks_like_scientific_notation_id("501e42"));
        assert!(
            !looks_like_scientific_notation_id("2a335e"),
            "hex with a non-e hex digit is safe"
        );
        assert!(!looks_like_scientific_notation_id("12345"));
        assert!(
            !looks_like_scientific_notation_id("\"652e07\""),
            "quoted ids are safe"
        );
        assert!(!looks_like_scientific_notation_id("wiki:tasks:652e07"));
    }

    #[test]
    fn test_inspect_frontmatter_health_scientific_notation() {
        let content = "---\ntitle: T\nid: 652e07\ntype: task\n---\n\nBody";
        let h = inspect_frontmatter_health(content, "652e07");
        assert_eq!(h.scientific_notation_id.as_deref(), Some("652e07"));
        assert!(!h.duplicate_blocks);
        assert!(h.id_mismatch.is_none());
    }

    #[test]
    fn test_inspect_frontmatter_health_quoted_id_ok() {
        let content = "---\ntitle: T\nid: \"652e07\"\ntype: task\n---\n\nBody";
        let h = inspect_frontmatter_health(content, "652e07");
        assert!(
            h.scientific_notation_id.is_none(),
            "quoted scientific-looking id must not be flagged"
        );
    }

    #[test]
    fn test_inspect_frontmatter_health_duplicate_blocks() {
        let content = "---\ntitle: T\nid: abc\n---\n---\ntitle: T\nid: abc\n---\n\nBody";
        let h = inspect_frontmatter_health(content, "abc");
        assert!(
            h.duplicate_blocks,
            "two complete frontmatter blocks must be flagged"
        );
        let single = "---\ntitle: T\n---\n\n# Heading\n\nSome --- horizontal rule\n";
        let h2 = inspect_frontmatter_health(single, "abc");
        assert!(
            !h2.duplicate_blocks,
            "a single block with body '---' is not duplicate"
        );
    }

    #[test]
    fn test_inspect_frontmatter_health_id_mismatch() {
        let content = "---\ntitle: T\nid: 6520000000.0\n---\n\nBody";
        let h = inspect_frontmatter_health(content, "652e07");
        assert!(
            h.id_mismatch.is_some(),
            "frontmatter id must match the filename stem"
        );
        let ok = "---\ntitle: T\nid: wiki:tasks:cli\n---\n\nBody";
        let h2 = inspect_frontmatter_health(ok, "cli");
        assert!(
            h2.id_mismatch.is_none(),
            "wiki:tasks: prefix must be normalized away"
        );
    }

    #[test]
    fn test_inspect_frontmatter_health_empty_block_does_not_panic() {
        let content = "---\n---\n\n---\n\n## Context\n\nBody";
        let h = inspect_frontmatter_health(content, "static-config-templates-no-substitution");
        assert!(h.scientific_notation_id.is_none());
        assert!(!h.duplicate_blocks);
        assert!(h.id_mismatch.is_none());
        assert_eq!(extract_raw_first_frontmatter(content), Some(""));
    }

    #[test]
    fn test_frontmatter_to_yaml_preserves_id_and_unknown() {
        let content = "---\ntitle: T\nid: 652e07\ntype: task\ncreatedAt: '2026-01-01'\nstatus: todo\n---\n\nBody";
        let (fm, _body) = extract_frontmatter(content);
        let fm = fm.expect("frontmatter must parse");
        assert_eq!(fm.id, "652e07", "id field must be modeled");
        let yaml = frontmatter_to_yaml(&fm);
        assert!(
            yaml.contains("id: \"652e07\""),
            "id must be re-emitted quoted, got: {}",
            yaml
        );
        assert!(
            yaml.contains("createdAt"),
            "custom fields must survive the struct round-trip, got: {}",
            yaml
        );
        let reparsed = extract_frontmatter(&format!("---\n{}---\n\nBody", yaml))
            .0
            .expect("re-serialized frontmatter must parse");
        assert_eq!(
            reparsed.id, "652e07",
            "id must survive the full round-trip as a string, got: {:?}",
            reparsed.id
        );
    }
}
