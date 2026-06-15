use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::engine::{EdgeType, PageStatus, PageType, SectionDoc, WikiPageMeta};

/// A single relation entry from frontmatter
#[derive(Debug, Deserialize)]
pub struct Relation {
    #[serde(rename = "type")]
    pub edge_type: String,
    pub target: String,
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
        _ => PageType::Concept,
    }
}

/// Parse page status from string
pub fn parse_page_status(s: &str) -> PageStatus {
    match s.to_lowercase().as_str() {
        "todo" => PageStatus::Todo,
        "in-progress" | "wip" => PageStatus::InProgress,
        "done" | "complete" => PageStatus::Done,
        "blocked" => PageStatus::Blocked,
        "cancelled" | "canceled" => PageStatus::Cancelled,
        "draft" => PageStatus::Draft,
        "reviewed" | "review" => PageStatus::Reviewed,
        "superseded" => PageStatus::Superseded,
        "approved" => PageStatus::Approved,
        _ => PageStatus::Draft,
    }
}

/// Parse edge type from string
pub fn parse_edge_type(s: &str) -> Result<EdgeType, String> {
    match s.to_lowercase().as_str() {
        "extends" => Ok(EdgeType::Extends),
        "implements" => Ok(EdgeType::Implements),
        "example_of" | "exampleof" => Ok(EdgeType::ExampleOf),
        "part_of" | "partof" => Ok(EdgeType::PartOf),
        "relates_to" | "relatesto" | "related" => Ok(EdgeType::RelatesTo),
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
        custom => Ok(EdgeType::Custom(Box::leak(custom.to_string().into_boxed_str()))),
    }
}

/// Compute SHA-256 hash of content
pub fn content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Build WikiPageMeta from a file path and its content
pub fn parse_wiki_page(file_path: &Path, content: &str) -> WikiPageMeta {
    let (fm, _body) = extract_frontmatter(content);
    let _sections = split_sections(_body);

    // Infer ID from path: "wiki/concepts/auth.md" → "wiki:concepts:auth"
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let id = rel_path
        .strip_suffix(".md")
        .unwrap_or(&rel_path)
        .replace('/', ":");

    let title = fm.as_ref().and_then(|f| f.title.clone()).unwrap_or_else(|| {
        // Fallback: derive from filename
        file_path
            .file_stem()
            .map(|s| s.to_string_lossy().replace('-', " "))
            .unwrap_or_default()
    });

    let page_type = fm.as_ref()
        .and_then(|f| f.page_type.as_deref())
        .map(parse_page_type)
        .unwrap_or(PageType::Concept);

    let status = fm.as_ref()
        .and_then(|f| f.status.as_deref())
        .map(parse_page_status)
        .unwrap_or(PageStatus::Draft);

    let _confidence = fm.as_ref()
        .and_then(|f| f.confidence.as_deref())
        .unwrap_or("medium")
        .to_string();

    let tags = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    let _aliases = fm.as_ref().map(|f| f.aliases.clone()).unwrap_or_default();
    let _sources = fm.as_ref().map(|f| f.sources.clone()).unwrap_or_default();

    WikiPageMeta {
        id,
        title,
        page_type,
        tags,
        status,
        assignee: None,
        path: file_path.to_path_buf(),
        created_at: String::new(),
        updated_at: String::new(),
    }
}

/// Build SectionDocs from parsed content
pub fn parse_sections(file_path: &Path, content: &str) -> Vec<SectionDoc> {
    let rel_path = file_path.to_string_lossy().replace('\\', "/");
    let page_id = rel_path
        .strip_suffix(".md")
        .unwrap_or(&rel_path)
        .replace('/', ":");

    let (_, body) = extract_frontmatter(content);
    let sections = split_sections(body);

    sections
        .into_iter()
        .map(|(header, body)| {
            let section_id = format!("{}#{}", page_id, header.to_lowercase().replace(' ', "-"));
            SectionDoc { section_id, page_id: page_id.clone(), header, body }
        })
        .collect()
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
}
