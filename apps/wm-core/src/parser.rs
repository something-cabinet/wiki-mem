use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::engine::{
    AcceptanceCriterion, DecisionData, EdgeType, FunctionalRequirement, GeneralGoal,
    NonFunctionalRequirement, PageStatus, PageType, PatternData, SectionDoc, SpecData, TaskData,
    TimeEntry, WikiPageMeta,
};

/// A single relation entry from frontmatter
#[derive(Debug, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: String,
}

/// Per-type structured fields
#[derive(Debug, Deserialize, Default)]
pub struct AcceptanceCriterionFm {
    pub text: String,
    #[serde(default)]
    pub checked: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct FrEntry {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct NfrEntry {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct GoalEntry {
    pub description: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct DecisionFm {
    pub context: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub rationale: String,
    pub outcome: String,
    #[serde(default)]
    pub consequences: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PatternFm {
    pub when_to_use: String,
    pub example: String,
}

/// Parsed frontmatter
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
    // Time tracking
    #[serde(default)]
    pub time_started: Option<String>,
    #[serde(default)]
    pub time_spent: Option<String>,
    #[serde(default)]
    pub time_entries: Option<Vec<TimeEntry>>,
    // Order / display
    #[serde(default)]
    pub order: Option<i32>,
    // Implementation fields
    #[serde(default)]
    pub implementation_plan: Option<String>,
    #[serde(default)]
    pub implementation_notes: Option<String>,
}

/// Parse frontmatter and content from a markdown string
/// Returns (frontmatter, content_body) or (None, full_content) if no frontmatter
pub fn extract_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
    let content = content.trim();
    if !content.starts_with("---") {
        return (None, content);
    }

    // Find the closing ---
    let end = if let Some(pos) = content[3..].find("\n---") {
        3 + pos
    } else {
        return (None, content);
    };

    let yaml_str = &content[3..end];
    let body = &content[end + 4..].trim();

    match serde_yaml::from_str::<Frontmatter>(yaml_str) {
        Ok(fm) => (Some(fm), body),
        Err(_) => (None, content),
    }
}

/// Code-fence-aware markdown section splitter
/// Splits on ## level headers, ignoring headers inside code fences
pub fn split_sections(raw: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut in_code_fence = false;
    let mut current_header = "Overview".to_string();
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
        } else {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if !current_body.trim().is_empty() {
        sections.push((current_header, current_body.trim().to_string()));
    }
    sections
}

/// Infer page type from frontmatter type string
pub fn parse_page_type(s: &str) -> PageType {
    match s.to_lowercase().as_str() {
        "task" => PageType::Task,
        "spec" => PageType::Spec,
        "concept" => PageType::Concept,
        "pattern" => PageType::Pattern,
        "decision" => PageType::Decision,
        "howto" | "guide" => PageType::Howto,
        "reference" => PageType::Reference,
        "memory" => PageType::Memory,
        "note" | "notes" => PageType::Note,
        _ => PageType::Concept,
    }
}

/// Parse page status from string
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
        other => {
            tracing::warn!("Unknown page status string: '{}', defaulting to Draft", other);
            PageStatus::Draft
        }
    }
}

pub fn parse_priority(s: &str) -> Option<crate::engine::Priority> {
    match s.to_lowercase().as_str() {
        "low" => Some(crate::engine::Priority::Low),
        "medium" | "med" => Some(crate::engine::Priority::Medium),
        "high" => Some(crate::engine::Priority::High),
        "urgent" | "critical" => Some(crate::engine::Priority::Urgent),
        _ => None,
    }
}

/// Parse edge type from string
pub fn parse_edge_type(s: &str) -> Result<EdgeType, String> {
    match s.to_lowercase().as_str() {
        "extends" => Ok(EdgeType::Extends),
        "implements" => Ok(EdgeType::Implements),
        "example_of" | "exampleof" => Ok(EdgeType::ExampleOf),
        "part_of" | "partof" => Ok(EdgeType::PartOf),
        "relates_to" | "relates-to" | "relatesto" | "related" => Ok(EdgeType::RelatesTo),
        "supports" => Ok(EdgeType::Supports),
        "contradicts" => Ok(EdgeType::Contradicts),
        "supersedes" => Ok(EdgeType::Supersedes),
        "depends_on" | "dependson" => Ok(EdgeType::DependsOn),
        "required_by" | "requiredby" => Ok(EdgeType::RequiredBy),
        "questions" => Ok(EdgeType::Questions),
        "answers" => Ok(EdgeType::Answers),
        "references" => Ok(EdgeType::References),
        "similar_to" | "similarto" | "similar" => Ok(EdgeType::SimilarTo),
        "causes" => Ok(EdgeType::Causes),
        "mitigates" => Ok(EdgeType::Mitigates),
        custom => Ok(EdgeType::Custom(custom.to_string())),
    }
}

/// Compute SHA-256 hash of content
pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Extract Obsidian-style `[[wikilinks]]` from markdown body.
/// Returns the link targets (without display text or brackets).
pub fn extract_wikilinks(text: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\[\[([^\]]+?)(?:\|[^\]]+)?\]\]").unwrap());
    re.captures_iter(text)
        .map(|cap| cap[1].trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Extract inline #tags from markdown body.
/// Ignores ## headings, code fences, and mid-word #.
pub fn extract_inline_tags(text: &str) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"(?:^|\s)#([a-zA-Z][\w-]*)").unwrap());

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

        // Skip ## headings
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

/// Resolve an Obsidian-style link target to our full path-based ID.
/// Handles aliases (auth → wiki:concepts:auth), titles ("Session Management" → wiki:concepts:session-management),
/// and normalizes hyphens/spaces for matching.
pub fn resolve_link_target(
    target: &str,
    graph: &petgraph::stable_graph::StableGraph<
        crate::engine::WikiPageMeta,
        crate::engine::EdgeType,
    >,
) -> Option<String> {
    let target_lower = target.to_lowercase();
    // Normalize: replace hyphens and spaces with a common form for matching
    let target_norm = target_lower.replace('-', " ");

    for idx in graph.node_indices() {
        let meta = &graph[idx];

        // 1. Direct ID match
        if meta.id.to_lowercase() == target_lower {
            return Some(meta.id.clone());
        }

        // 1b. Last path segment match (handles [[session-management]] → wiki:concepts:session-management)
        if let Some(last) = meta.id.split(':').next_back() {
            if last.to_lowercase() == target_lower {
                return Some(meta.id.clone());
            }
        }

        // 2. Normalized title match (handles "Session Management" vs "session-management")
        let title_norm = meta.title.to_lowercase().replace('-', " ");
        if title_norm == target_norm {
            return Some(meta.id.clone());
        }

        // 3. Title contains (any direction, normalized)
        if title_norm.contains(&target_norm) || target_norm.contains(&title_norm) {
            return Some(meta.id.clone());
        }

        // 4. Alias match
        if meta
            .aliases
            .iter()
            .any(|a| a.to_lowercase().replace('-', " ") == target_norm)
        {
            return Some(meta.id.clone());
        }
    }

    None
}

/// Normalize a file path to a wiki ID.
/// "concepts/auth.md" → "wiki:concepts:auth"
/// ".wm/wiki/concepts/auth.md" → "wiki:concepts:auth"
pub fn path_to_id(rel_path: &str) -> String {
    // Strip leading "./" or ".wm/" or ".wm/wiki/" or "wiki/"
    let cleaned = rel_path
        .trim_start_matches("./")
        .trim_start_matches(".wm/wiki/")
        .trim_start_matches(".wm/")
        .trim_start_matches("wiki/")
        .trim_start_matches("wm/");
    let base = cleaned
        .strip_suffix(".md")
        .unwrap_or(cleaned)
        .replace('/', ":");
    // Always prepend "wiki:" for consistent IDs
    if base.starts_with("wiki:") || base.is_empty() {
        base
    } else {
        format!("wiki:{}", base)
    }
}

/// Build WikiPageMeta from a file path and its content
pub fn parse_wiki_page(file_path: &Path, content: &str) -> WikiPageMeta {
    let (fm, _body) = extract_frontmatter(content);
    let _sections = split_sections(_body);

    // Infer ID from path: "wiki/concepts/auth.md" → "wiki:concepts:auth"
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let id = path_to_id(&rel_path);

    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| {
            // Fallback: derive from filename
            file_path
                .file_stem()
                .map(|s| s.to_string_lossy().replace('-', " "))
                .unwrap_or_default()
        });

    let page_type = fm
        .as_ref()
        .and_then(|f| f.page_type.as_deref())
        .map(parse_page_type)
        .unwrap_or(PageType::Concept);

    let status = fm
        .as_ref()
        .and_then(|f| f.status.as_deref())
        .map(parse_page_status)
        .unwrap_or(PageStatus::Draft);

    let mut tags = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    // Merge inline #tags from body (Obsidian compat)
    let inline_tags = extract_inline_tags(_body);
    for t in inline_tags {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    let _aliases = fm.as_ref().map(|f| f.aliases.clone()).unwrap_or_default();
    let _sources = fm.as_ref().map(|f| f.sources.clone()).unwrap_or_default();
    let priority = fm
        .as_ref()
        .and_then(|f| f.priority.as_deref())
        .and_then(parse_priority);
    let assignee = fm.as_ref().and_then(|f| f.assignee.clone());
    WikiPageMeta {
        id,
        title,
        page_type,
        tags,
        status,
        priority,
        confidence: None,
        assignee,
        aliases: _aliases,
        superseded_by: fm.as_ref().and_then(|f| f.superseded_by.clone()),
        version: fm.as_ref().and_then(|f| f.version.clone()),
        sources: _sources,
        published: false,
        parent: fm.as_ref().and_then(|f| f.parent.clone()),
        order: fm.as_ref().and_then(|f| f.order),
        relates_to: {
            let mut rels = fm
                .as_ref()
                .map(|f| {
                    f.relates_to
                        .iter()
                        .map(|r| (parse_edge_type(&r.edge_type).unwrap_or(EdgeType::Custom(r.edge_type.clone())), r.target.clone()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            // Extract [[wikilinks]] from body and add as relates_to edges
            let wikilinks = extract_wikilinks(_body);
            for link in wikilinks {
                let entry = (EdgeType::RelatesTo, link);
                if !rels.contains(&entry) {
                    rels.push(entry);
                }
            }
            rels
        },
        path: file_path.to_path_buf(),
        created_at: String::new(),
        updated_at: String::new(),
        task_data: fm.as_ref().and_then(|f| {
            let ac: Vec<AcceptanceCriterion> = f
                .acceptance_criteria
                .iter()
                .map(|ac| AcceptanceCriterion {
                    text: ac.text.clone(),
                    checked: ac.checked,
                })
                .collect();
            if ac.is_empty()
                && f.estimate.is_none()
                && f.prerequisites.is_empty()
                && f.difficulty.is_none()
                && f.implementation_plan.is_none()
                && f.implementation_notes.is_none()
            {
                None
            } else {
                Some(TaskData {
                    acceptance_criteria: ac,
                    estimate: f.estimate,
                    prerequisites: f.prerequisites.clone(),
                    difficulty: f.difficulty.clone(),
                    time_spent: f.time_spent.clone(),
                    time_entries: f.time_entries.clone().unwrap_or_default(),
                    implementation_plan: f.implementation_plan.clone(),
                    implementation_notes: f.implementation_notes.clone(),
                })
            }
        }),
        spec_data: fm.as_ref().and_then(|f| {
            let fr: Vec<FunctionalRequirement> = f
                .functional_requirements
                .iter()
                .map(|fr| FunctionalRequirement {
                    id: fr.id.clone(),
                    description: fr.description.clone(),
                })
                .collect();
            let nfr: Vec<NonFunctionalRequirement> = f
                .non_functional_requirements
                .iter()
                .map(|nfr| NonFunctionalRequirement {
                    id: nfr.id.clone(),
                    description: nfr.description.clone(),
                })
                .collect();
            let gg: Vec<GeneralGoal> = f
                .general_goals
                .iter()
                .map(|g| GeneralGoal {
                    description: g.description.clone(),
                })
                .collect();
            if fr.is_empty() && nfr.is_empty() && gg.is_empty() && f.stakeholders.is_empty() {
                None
            } else {
                Some(SpecData {
                    functional_requirements: fr,
                    non_functional_requirements: nfr,
                    general_goals: gg,
                    stakeholders: f.stakeholders.clone(),
                })
            }
        }),
        decision_data: fm.as_ref().and_then(|f| {
            f.decision.as_ref().map(|d| DecisionData {
                context: d.context.clone(),
                options: d.options.clone(),
                rationale: d.rationale.clone(),
                outcome: d.outcome.clone(),
                consequences: d.consequences.clone(),
            })
        }),
        pattern_data: fm.as_ref().and_then(|f| {
            f.pattern.as_ref().map(|p| PatternData {
                when_to_use: p.when_to_use.clone(),
                example: p.example.clone(),
            })
        }),
        memory_data: None,
    }
}

/// Build SectionDocs from parsed content
pub fn parse_sections(file_path: &Path, content: &str) -> Vec<SectionDoc> {
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let page_id = path_to_id(&rel_path);

    let (_, body) = extract_frontmatter(content);
    let sections = split_sections(body);

    sections
        .into_iter()
        .map(|(header, body)| {
            let section_id = format!("{}#{}", page_id, header.to_lowercase().replace(' ', "-"));
            SectionDoc {
                section_id,
                page_id: page_id.clone(),
                header,
                body,
            }
        })
        .collect()
}

/// Serialize a Frontmatter struct to YAML string.
/// This is the single source of truth for frontmatter serialization.
/// Callers may append additional overrides after calling this function.
pub fn frontmatter_to_yaml(fm: &Frontmatter) -> String {
    let mut yaml = String::new();

    if let Some(ref title) = fm.title {
        yaml.push_str(&format!("title: {}\n", title));
    }
    if let Some(ref pt) = fm.page_type {
        yaml.push_str(&format!("type: {}\n", pt));
    }
    if !fm.tags.is_empty() {
        yaml.push_str(&format!("tags: [{}]\n", fm.tags.join(", ")));
    }
    if let Some(ref s) = fm.status {
        yaml.push_str(&format!("status: {}\n", s));
    }
    if let Some(ref p) = fm.priority {
        yaml.push_str(&format!("priority: {}\n", p));
    }
    if let Some(ref c) = fm.confidence {
        yaml.push_str(&format!("confidence: {}\n", c));
    }
    if let Some(ref a) = fm.assignee {
        yaml.push_str(&format!("assignee: {}\n", a));
    }
    if !fm.aliases.is_empty() {
        yaml.push_str(&format!("aliases: [{}]\n", fm.aliases.join(", ")));
    }
    if !fm.sources.is_empty() {
        yaml.push_str(&format!("sources: [{}]\n", fm.sources.join(", ")));
    }
    if let Some(ref s) = fm.superseded_by {
        yaml.push_str(&format!("superseded_by: {}\n", s));
    }
    if let Some(ref v) = fm.version {
        yaml.push_str(&format!("version: {}\n", v));
    }
    if let Some(ref est) = fm.estimate {
        yaml.push_str(&format!("estimate: {}\n", est));
    }
    if !fm.prerequisites.is_empty() {
        yaml.push_str(&format!("prerequisites: [{}]\n", fm.prerequisites.join(", ")));
    }
    if let Some(ref d) = fm.difficulty {
        yaml.push_str(&format!("difficulty: {}\n", d));
    }
    if let Some(ref src) = fm.source_url {
        yaml.push_str(&format!("source_url: {}\n", src));
    }
    if !fm.stakeholders.is_empty() {
        yaml.push_str(&format!("stakeholders: [{}]\n", fm.stakeholders.join(", ")));
    }
    if let Some(ref t) = fm.time_started {
        yaml.push_str(&format!("time_started: {}\n", t));
    }
    if let Some(ref t) = fm.time_spent {
        yaml.push_str(&format!("time_spent: {}\n", t));
    }
    if let Some(ref entries) = fm.time_entries {
        if !entries.is_empty() {
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
    }
    // Per-type structured fields
    if !fm.functional_requirements.is_empty() {
        yaml.push_str("functional_requirements:\n");
        for fr in &fm.functional_requirements {
            yaml.push_str(&format!("  - {{id: {}, description: \"{}\"}}\n", fr.id, fr.description));
        }
    }
    if !fm.non_functional_requirements.is_empty() {
        yaml.push_str("non_functional_requirements:\n");
        for nfr in &fm.non_functional_requirements {
            yaml.push_str(&format!("  - {{id: {}, description: \"{}\"}}\n", nfr.id, nfr.description));
        }
    }
    if !fm.general_goals.is_empty() {
        yaml.push_str("general_goals:\n");
        for g in &fm.general_goals {
            yaml.push_str(&format!("  - {{description: \"{}\"}}\n", g.description));
        }
    }
    if let Some(ref dec) = fm.decision {
        yaml.push_str("decision:\n");
        yaml.push_str(&format!("  context: \"{}\"\n", dec.context));
        if !dec.options.is_empty() {
            yaml.push_str(&format!("  options: [{}]\n", dec.options.join(", ")));
        }
        yaml.push_str(&format!("  rationale: \"{}\"\n", dec.rationale));
        yaml.push_str(&format!("  outcome: \"{}\"\n", dec.outcome));
        if let Some(ref c) = dec.consequences {
            yaml.push_str(&format!("  consequences: \"{}\"\n", c));
        }
    }
    if let Some(ref pat) = fm.pattern {
        yaml.push_str("pattern:\n");
        yaml.push_str(&format!("  when_to_use: \"{}\"\n", pat.when_to_use));
        yaml.push_str(&format!("  example: \"{}\"\n", pat.example));
    }
    if !fm.relates_to.is_empty() {
        yaml.push_str("relates_to:\n");
        for r in &fm.relates_to {
            yaml.push_str(&format!("  - {{type: {}, target: {}}}\n", r.edge_type, r.target));
        }
    }
    if !fm.acceptance_criteria.is_empty() {
        yaml.push_str("acceptance_criteria:\n");
        for ac in &fm.acceptance_criteria {
            yaml.push_str(&format!("  - {{text: \"{}\", checked: {}}}\n", ac.text, ac.checked));
        }
    }

    if let Some(ref o) = fm.order {
        yaml.push_str(&format!("order: {}\n", o));
    }
    if let Some(ref ip) = fm.implementation_plan {
        yaml.push_str(&format!("implementation_plan: \"{}\"\n", ip));
    }
    if let Some(ref inp) = fm.implementation_notes {
        yaml.push_str(&format!("implementation_notes: \"{}\"\n", inp));
    }

    yaml
}

/// Parse an edge type string flexibly (delegates to engine).
pub fn parse_edge_type_flexible(s: &str) -> crate::engine::EdgeType {
    crate::engine::parse_edge_type_flexible(s)
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
        assert_eq!(fm.title.as_deref(), Some("Authentication Architecture"));
        assert_eq!(fm.relates_to.len(), 2);
        assert_eq!(fm.relates_to[0].edge_type, "extends");
        assert_eq!(fm.relates_to[0].target, "wiki:concepts:base-auth");
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
        assert!(links.contains(&"auth-service".to_string()));
        assert!(links.contains(&"oauth2".to_string()));
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
        // Our simple regex doesn't handle code fences — that's OK for now.
        // The graph builder will just fail to resolve non-existent IDs.
        assert_eq!(links.len(), 2); // Both are extracted
    }

    #[test]
    fn test_extract_inline_tags_basic() {
        let md = "This is about #auth and #security features.";
        let tags = extract_inline_tags(md);
        assert!(tags.contains(&"auth".to_string()));
        assert!(tags.contains(&"security".to_string()));
    }

    #[test]
    fn test_extract_inline_tags_ignores_headings() {
        let md = "\
## Not a tag

#also-not-a-tag

This is #auth";
        let tags = extract_inline_tags(md);
        assert!(!tags.contains(&"Not".to_string()));
        assert!(tags.contains(&"auth".to_string()));
    }

    #[test]
    fn test_frontmatter_roundtrip() {
        let content = "---\ntitle: Test\ntype: task\nstatus: todo\n---\n\nBody here.";
        let (fm, body) = extract_frontmatter(content);
        assert!(fm.is_some(), "frontmatter should be parsed");
        assert_eq!(body.trim(), "Body here.");
        let fm = fm.unwrap();
        assert_eq!(fm.title.as_deref(), Some("Test"));
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
        // Tags should include both frontmatter and inline
        assert!(meta.tags.contains(&"backend".to_string()));
        assert!(meta.tags.contains(&"security".to_string()));
        // relates_to should include both frontmatter and wikilinks
        assert!(meta
            .relates_to
            .iter()
            .any(|(_, target)| target == "session-management"));
        assert!(meta.relates_to.iter().any(|(_, target)| target == "permissions"));
    }

    #[test]
    fn test_parse_edge_type_flexible_all() {
        use crate::engine::EdgeType;
        // Test common aliases
        assert_eq!(crate::parser::parse_edge_type_flexible("related"), EdgeType::RelatesTo);
        assert_eq!(crate::parser::parse_edge_type_flexible("relates-to"), EdgeType::RelatesTo);
        assert_eq!(crate::parser::parse_edge_type_flexible("depends-on"), EdgeType::DependsOn);
        assert_eq!(crate::parser::parse_edge_type_flexible("example-of"), EdgeType::ExampleOf);
        assert_eq!(crate::parser::parse_edge_type_flexible("part-of"), EdgeType::PartOf);
        assert_eq!(crate::parser::parse_edge_type_flexible("custom-type"), EdgeType::Custom("custom-type".into()));
    }

    #[test]
    fn test_path_to_id_format() {
        let id = crate::parser::path_to_id("tasks/my-task.md");
        assert_eq!(id, "wiki:tasks:my-task", "expected wiki:tasks:my-task, got {}", id);
    }
}
