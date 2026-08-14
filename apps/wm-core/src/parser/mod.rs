use sha2::{Digest, Sha256};
use std::path::Path;

use wm_engine::status::PageStatus;
use wm_engine::{
    AcceptanceCriterion, DecisionData, EdgeType, FunctionalRequirement, GeneralGoal,
    NonFunctionalRequirement, PageType, PatternData, RuleData, SectionDoc, SpecData, TaskData,
    WikiPageMeta,
};

pub mod models;
pub use models::*;

pub fn extract_frontmatter(content: &str) -> (Option<Frontmatter>, &str) {
    extract_frontmatter_from("<unknown source>", content)
}

pub fn extract_raw_frontmatter(content: &str) -> (Option<String>, &str) {
    let content = content.trim();
    if !content.starts_with("---") {
        return (None, content);
    }
    let Some(pos) = content[3..].find("\n---") else {
        return (None, content);
    };
    let end = 3usize.wrapping_add(pos);
    let yaml_str = &content[4..end.wrapping_add(1)];
    let body = &content[end.wrapping_add(4)..].trim();
    (Some(yaml_str.to_string()), body)
}

pub fn extract_frontmatter_from<'a>(
    source: &str,
    content: &'a str,
) -> (Option<Frontmatter>, &'a str) {
    let content = content.trim();
    if !content.starts_with("---") {
        return (None, content);
    }

    let Some(pos) = content[3..].find("\n---") else {
        return (None, content);
    };
    let end = 3usize.wrapping_add(pos);
    let yaml_str = &content[3..end];
    let body = &content[end.wrapping_add(4)..].trim();

    match serde_yaml::from_str::<Frontmatter>(yaml_str) {
        Ok(fm) => (Some(fm), body),
        Err(e) => {
            tracing::warn!(
                "Frontmatter parse error in {}: {} — frontmatter was: {}",
                source,
                e,
                yaml_str.chars().take(160).collect::<String>()
            );
            (None, content)
        }
    }
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
        "rule" => PageType::Rule,
        "core" => PageType::Core,
        _ => PageType::Concept,
    }
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

pub fn parse_priority(s: &str) -> Option<wm_engine::status::Priority> {
    match s.to_lowercase().as_str() {
        "low" => Some(wm_engine::status::Priority::Low),
        "medium" | "med" => Some(wm_engine::status::Priority::Medium),
        "high" => Some(wm_engine::status::Priority::High),
        "urgent" | "critical" => Some(wm_engine::status::Priority::Urgent),
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

/// All nodes matching `target` under the same fuzzy rules as
/// `resolve_link_target`, in graph node order. Callers use the length to detect
/// ambiguous resolution (multiple candidate targets → `EdgeProvenance::Ambiguous`).
pub fn resolve_link_target_candidates(
    target: &str,
    graph: &petgraph::stable_graph::StableGraph<wm_engine::WikiPageMeta, wm_engine::GraphEdge>,
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
    graph: &petgraph::stable_graph::StableGraph<wm_engine::WikiPageMeta, wm_engine::GraphEdge>,
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
    if base.starts_with("wiki:") || base.is_empty() {
        base
    } else {
        format!("wiki:{}", base)
    }
}

pub fn parse_wiki_page(file_path: &Path, content: &str) -> WikiPageMeta {
    let (mut fm, _body) = extract_frontmatter_from(&file_path.to_string_lossy(), content);
    let _sections = split_sections(_body);

    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let id = path_to_id(&rel_path);

    let title = fm.as_mut().and_then(|f| f.title.take()).unwrap_or_else(|| {
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

    let mut tags = fm
        .as_mut()
        .map(|f| std::mem::take(&mut f.tags))
        .unwrap_or_default();
    let inline_tags = extract_inline_tags(_body);
    for t in inline_tags {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    let _aliases = fm
        .as_mut()
        .map(|f| std::mem::take(&mut f.aliases))
        .unwrap_or_default();
    let _sources = fm
        .as_mut()
        .map(|f| std::mem::take(&mut f.sources))
        .unwrap_or_default();
    let priority = fm
        .as_ref()
        .and_then(|f| f.priority.as_deref())
        .and_then(parse_priority);
    let assignee = fm.as_mut().and_then(|f| f.assignee.take());
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
        superseded_by: fm.as_mut().and_then(|f| f.superseded_by.take()),
        version: fm.as_mut().and_then(|f| f.version.take()),
        sources: _sources,
        published: false,
        parent: fm.as_mut().and_then(|f| f.parent.take()),
        order: fm.as_ref().and_then(|f| f.order),
        relates_to: {
            let mut rels = fm
                .as_mut()
                .map(|f| {
                    std::mem::take(&mut f.relates_to)
                        .into_iter()
                        .map(|r| (EdgeType::from_str_flexible(&r.edge_type), r.target))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
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
        task_data: fm.as_mut().and_then(|f| {
            let ac: Vec<AcceptanceCriterion> = std::mem::take(&mut f.acceptance_criteria)
                .into_iter()
                .map(|ac| AcceptanceCriterion {
                    text: ac.text,
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
                    prerequisites: std::mem::take(&mut f.prerequisites),
                    difficulty: f.difficulty.take(),
                    time_spent: f.time_spent.take(),
                    time_entries: f.time_entries.take().unwrap_or_default(),
                    implementation_plan: f.implementation_plan.take(),
                    implementation_notes: f.implementation_notes.take(),
                })
            }
        }),
        spec_data: fm.as_mut().and_then(|f| {
            let fr: Vec<FunctionalRequirement> = std::mem::take(&mut f.functional_requirements)
                .into_iter()
                .map(|fr| FunctionalRequirement {
                    id: fr.id,
                    description: fr.description,
                })
                .collect();
            let nfr: Vec<NonFunctionalRequirement> =
                std::mem::take(&mut f.non_functional_requirements)
                    .into_iter()
                    .map(|nfr| NonFunctionalRequirement {
                        id: nfr.id,
                        description: nfr.description,
                    })
                    .collect();
            let gg: Vec<GeneralGoal> = std::mem::take(&mut f.general_goals)
                .into_iter()
                .map(|g| GeneralGoal {
                    description: g.description,
                })
                .collect();
            if fr.is_empty() && nfr.is_empty() && gg.is_empty() && f.stakeholders.is_empty() {
                None
            } else {
                Some(SpecData {
                    functional_requirements: fr,
                    non_functional_requirements: nfr,
                    general_goals: gg,
                    stakeholders: std::mem::take(&mut f.stakeholders),
                })
            }
        }),
        decision_data: fm.as_mut().and_then(|f| {
            f.decision.take().map(|d| DecisionData {
                context: d.context,
                options: d.options,
                rationale: d.rationale,
                outcome: d.outcome,
                consequences: d.consequences,
            })
        }),
        pattern_data: fm.as_mut().and_then(|f| {
            f.pattern.take().map(|p| PatternData {
                when_to_use: p.when_to_use,
                example: p.example,
            })
        }),
        memory_data: None,
        rule_data: fm.as_mut().and_then(|f| {
            f.category.take().map(|c| RuleData {
                category: c,
                rationale: f.rationale.take().unwrap_or_default(),
                example: f.example.take(),
                anti_pattern: f.anti_pattern.take(),
            })
        }),
    }
}

pub fn parse_sections(file_path: &Path, content: &str) -> Vec<SectionDoc> {
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let page_id = path_to_id(&rel_path);

    let (fm, body) = extract_frontmatter_from(&rel_path, content);
    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| {
            file_path
                .file_stem()
                .map(|s| s.to_string_lossy().replace('-', " "))
                .unwrap_or_default()
        });
    let mut tags: Vec<String> = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    let inline_tags = extract_inline_tags(body);
    for t in inline_tags {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }

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
                title: title.clone(),
                tags: tags.clone(),
            }
        })
        .collect()
}

pub fn frontmatter_to_yaml(fm: &Frontmatter) -> String {
    let mut yaml = String::new();

    append_scalar_fields(&mut yaml, fm);
    append_time_entries(&mut yaml, fm);
    append_requirements(&mut yaml, fm);
    append_decision(&mut yaml, fm);
    append_pattern(&mut yaml, fm);
    append_relates_to(&mut yaml, fm);
    append_acceptance_criteria(&mut yaml, fm);
    append_rule_fields(&mut yaml, fm);
    append_unknown_fields(&mut yaml, fm);

    yaml
}

fn append_scalar_fields(yaml: &mut String, fm: &Frontmatter) {
    if let Some(ref title) = fm.title {
        yaml.push_str(&format!(
            "title: {}\n",
            crate::page::helpers::yaml_helper::yaml_scalar(title)
        ));
    }
    if let Some(ref id) = fm.id {
        yaml.push_str(&format!(
            "id: {}\n",
            crate::page::helpers::yaml_helper::yaml_quote(id)
        ));
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
        yaml.push_str(&format!(
            "prerequisites: [{}]\n",
            fm.prerequisites.join(", ")
        ));
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
    if let Some(ref o) = fm.order {
        yaml.push_str(&format!("order: {}\n", o));
    }
    if let Some(ref ip) = fm.implementation_plan {
        yaml.push_str(&format!("implementation_plan: \"{}\"\n", ip));
    }
    if let Some(ref inp) = fm.implementation_notes {
        yaml.push_str(&format!("implementation_notes: \"{}\"\n", inp));
    }
}

fn append_time_entries(yaml: &mut String, fm: &Frontmatter) {
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
}

fn append_requirements(yaml: &mut String, fm: &Frontmatter) {
    if !fm.functional_requirements.is_empty() {
        yaml.push_str("functional_requirements:\n");
        for fr in &fm.functional_requirements {
            yaml.push_str(&format!(
                "  - {{id: {}, description: \"{}\"}}\n",
                fr.id, fr.description
            ));
        }
    }
    if !fm.non_functional_requirements.is_empty() {
        yaml.push_str("non_functional_requirements:\n");
        for nfr in &fm.non_functional_requirements {
            yaml.push_str(&format!(
                "  - {{id: {}, description: \"{}\"}}\n",
                nfr.id, nfr.description
            ));
        }
    }
    if !fm.general_goals.is_empty() {
        yaml.push_str("general_goals:\n");
        for g in &fm.general_goals {
            yaml.push_str(&format!("  - {{description: \"{}\"}}\n", g.description));
        }
    }
}

fn append_decision(yaml: &mut String, fm: &Frontmatter) {
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
}

fn append_pattern(yaml: &mut String, fm: &Frontmatter) {
    if let Some(ref pat) = fm.pattern {
        yaml.push_str("pattern:\n");
        yaml.push_str(&format!("  when_to_use: \"{}\"\n", pat.when_to_use));
        yaml.push_str(&format!("  example: \"{}\"\n", pat.example));
    }
}

fn append_relates_to(yaml: &mut String, fm: &Frontmatter) {
    if !fm.relates_to.is_empty() {
        yaml.push_str("relates_to:\n");
        for r in &fm.relates_to {
            yaml.push_str(&format!(
                "  - {{type: {}, target: {}}}\n",
                r.edge_type, r.target
            ));
        }
    }
}

fn append_acceptance_criteria(yaml: &mut String, fm: &Frontmatter) {
    if !fm.acceptance_criteria.is_empty() {
        yaml.push_str("acceptance_criteria:\n");
        for ac in &fm.acceptance_criteria {
            yaml.push_str(&format!(
                "  - {{text: \"{}\", checked: {}}}\n",
                ac.text, ac.checked
            ));
        }
    }
}

fn append_rule_fields(yaml: &mut String, fm: &Frontmatter) {
    if let Some(ref cat) = fm.category {
        yaml.push_str(&format!("category: {:?}\n", cat));
    }
    if let Some(ref rat) = fm.rationale {
        yaml.push_str(&format!("rationale: \"{}\"\n", rat));
    }
    if let Some(ref ex) = fm.example {
        yaml.push_str(&format!("example: \"{}\"\n", ex));
    }
    if let Some(ref ap) = fm.anti_pattern {
        yaml.push_str(&format!("anti_pattern: \"{}\"\n", ap));
    }
}

fn append_unknown_fields(yaml: &mut String, fm: &Frontmatter) {
    for (key, value) in &fm.unknown {
        if key == "id" {
            continue;
        }
        let rendered = serde_yaml::to_string(&value).unwrap_or_default();
        let rendered = rendered.trim_end();
        if rendered.contains('\n') {
            yaml.push_str(&format!("{}:\n", key));
            for line in rendered.lines() {
                yaml.push_str(&format!("  {}\n", line));
            }
        } else {
            yaml.push_str(&format!("{}: {}\n", key, rendered));
        }
    }
}

/// How many complete `---`-delimited YAML blocks sit at the top of a file.
/// Two or more means the file's frontmatter was duplicated by a buggy write.
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

/// Extract the raw YAML of the first frontmatter block, if present. Returns
/// `Some("")` for an empty block (`---\n---`).
pub fn extract_raw_first_frontmatter(content: &str) -> Option<&str> {
    let rest = content.trim_start();
    if !rest.starts_with("---") {
        return None;
    }
    let pos = rest[3..].find("\n---")?;
    let end = 3usize.wrapping_add(pos);
    if end < 4 {
        return Some("");
    }
    Some(&rest[4..end])
}

/// Extract the value of a top-level `id:` line from raw frontmatter (quotes
/// stripped). Returns None when the block has no `id` line.
pub fn frontmatter_id_from_raw(raw_fm: &str) -> Option<String> {
    frontmatter_id_raw_from_raw(raw_fm).map(|v| {
        let v = v.trim();
        v.trim_matches('"').trim_matches('\'').to_string()
    })
}

/// Extract the RAW (unquoted-ness preserved) value of a top-level `id:` line.
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

/// `^[0-9]+e[0-9]+$` — an unquoted YAML value that serde_yaml reads as a
/// scientific-notation float (e.g. a 6-hex-char page id like `652e07`), which
/// gets rewritten to `6520000000.0` on the next frontmatter round-trip.
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
        } else if !c.is_ascii_digit() {
            return false;
        }
    }
    has_e
}

/// Frontmatter consistency facts used by the validator/lint tools.
pub struct FrontmatterHealth {
    /// An unquoted `id:` value matching `^[0-9]+e[0-9]+$` — will be corrupted
    /// by any YAML round-trip.
    pub scientific_notation_id: Option<String>,
    /// More than one complete `---` block at the top of the file.
    pub duplicate_blocks: bool,
    /// Frontmatter `id` doesn't match the filename stem (task pages only).
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
        let normalized = if let Some(rest) = fid.strip_prefix("wiki:tasks:") {
            rest
        } else if let Some(rest) = fid.strip_prefix("wiki:") {
            rest
        } else {
            fid
        };
        if !filename_stem.is_empty() && normalized != filename_stem {
            Some(format!(
                "Frontmatter id '{}' does not match filename stem '{}'",
                fid, filename_stem
            ))
        } else {
            None
        }
    });

    FrontmatterHealth {
        scientific_notation_id,
        duplicate_blocks: blocks >= 2,
        id_mismatch,
    }
}

pub fn parse_edge_type_flexible(s: &str) -> wm_engine::EdgeType {
    wm_engine::EdgeType::from_str_flexible(s)
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
    fn test_parse_edge_type_flexible_all() {
        use wm_engine::EdgeType;
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
        // An empty frontmatter block followed by a markdown hr is legal (but
        // odd) — the validator must not panic on it.
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
        assert_eq!(fm.id.as_deref(), Some("652e07"), "id field must be modeled");
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
            reparsed.id.as_deref(),
            Some("652e07"),
            "id must survive the full round-trip as a string, got: {:?}",
            reparsed.id
        );
    }
}
