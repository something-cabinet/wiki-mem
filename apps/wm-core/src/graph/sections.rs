use rayon::prelude::*;
use std::borrow::Cow;
use std::path::Path;
use tracing::warn;
use wm_engine::record_state_text;

use crate::engine::SectionDoc;
use crate::parser::{extract_frontmatter, extract_inline_tags, path_to_id, split_sections};

const FRONTMATTER_SECTION: &str = "frontmatter";
const FRONTMATTER_HEADER: &str = "Frontmatter";

pub fn build_sections_from_file(path: &Path) -> Option<Vec<SectionDoc>> {
    if !path.extension().map(|ext| ext == "md").unwrap_or(false) {
        return None;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to read wiki file {}: {}", path.display(), e);
            return None;
        }
    };

    let path_str = path.to_string_lossy().replace('\\', "/");
    let rel_path = path_str.split(".wm/wiki/").last().unwrap_or(&path_str);
    let page_id = path_to_id(rel_path);

    let (fm, body) = extract_frontmatter(&content);
    let searchable: Cow<'_, str> = record_state_text(body)
        .map(Cow::Owned)
        .unwrap_or(Cow::Borrowed(body));
    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().replace('-', " "))
                .unwrap_or_default()
        });
    let mut tags: Vec<String> = fm.as_ref().map(|f| f.tags.clone()).unwrap_or_default();
    let inline_tags = extract_inline_tags(&searchable);
    for t in inline_tags {
        if !tags.contains(&t) {
            tags.push(t);
        }
    }

    let sections: Vec<SectionDoc> = split_sections(&searchable)
        .into_iter()
        .map(|(header, body_text)| {
            let section_id = format!("{}#{}", page_id, header.to_lowercase().replace(' ', "-"));
            SectionDoc {
                section_id,
                page_id: page_id.clone(),
                header,
                body: body_text,
                title: title.clone(),
                tags: tags.clone(),
            }
        })
        .collect();

    let mut sections = sections;
    if let Some(frontmatter) = fm.as_ref() {
        let structured = structured_fields_text(frontmatter);
        if !structured.is_empty() {
            sections.push(SectionDoc {
                section_id: format!("{}#{FRONTMATTER_SECTION}", page_id),
                page_id: page_id.clone(),
                header: FRONTMATTER_HEADER.to_owned(),
                body: structured,
                title: title.clone(),
                tags: tags.clone(),
            });
        }
    }

    if sections.is_empty() {
        warn!(
            "No sections found in {} (empty or unparseable)",
            path.display()
        );
        return None;
    }

    Some(sections)
}

fn structured_fields_text(frontmatter: &crate::parser::Frontmatter) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.extend(frontmatter.aliases.iter().cloned());
    parts.extend(
        frontmatter
            .functional_requirements
            .iter()
            .map(|entry| format!("{} {}", entry.id, entry.description)),
    );
    parts.extend(
        frontmatter
            .non_functional_requirements
            .iter()
            .map(|entry| format!("{} {}", entry.id, entry.description)),
    );
    parts.extend(
        frontmatter
            .general_goals
            .iter()
            .map(|entry| entry.description.clone()),
    );
    parts.extend(
        frontmatter
            .acceptance_criteria
            .iter()
            .map(|entry| entry.text.clone()),
    );
    if let Some(decision) = frontmatter.decision.as_ref() {
        parts.push(decision.context.clone());
        parts.extend(decision.options.iter().cloned());
        parts.push(decision.rationale.clone());
        parts.push(decision.outcome.clone());
        if let Some(consequences) = decision.consequences.as_ref() {
            parts.push(consequences.clone());
        }
    }
    if let Some(pattern) = frontmatter.pattern.as_ref() {
        parts.push(pattern.when_to_use.clone());
        parts.push(pattern.example.clone());
    }
    if let Some(rationale) = frontmatter.rationale.as_ref() {
        parts.push(rationale.clone());
    }
    if let Some(example) = frontmatter.example.as_ref() {
        parts.push(example.clone());
    }
    if let Some(anti_pattern) = frontmatter.anti_pattern.as_ref() {
        parts.push(anti_pattern.clone());
    }
    parts.retain(|part| !part.trim().is_empty());
    parts.join("\n")
}

pub fn build_sections_from_wiki(wiki_dir: &Path) -> Vec<SectionDoc> {
    let paths: Vec<_> = walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name != "index.md" && name != "log.md"
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    paths
        .par_iter()
        .filter_map(|path| build_sections_from_file(path))
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    const RECORD_PAGE: &str = "---\ntitle: Choose a database\ntype: decision\n---\n\nschema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite over Postgres for the local store.\n\n  ## Rationale\n\n  Embedded, zero-ops, #storage.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected]\nanswers:\n  outcome: adopted\n";

    fn write_page(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_build_sections_from_file_exists() {
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../.wm/wiki/specs/graph-connectivity-fix.md"
        ));
        let result = build_sections_from_file(path);
        assert!(
            result.is_some(),
            "Expected Some(sections) for an existing wiki file"
        );
        let sections = result.unwrap();
        assert!(!sections.is_empty(), "Expected non-empty sections");
    }

    #[test]
    fn test_build_sections_from_file_nonexistent() {
        let path = Path::new("/tmp/this-file-does-not-exist-12345.md");
        let result = build_sections_from_file(path);
        assert!(result.is_none(), "Expected None for a nonexistent file");
    }

    #[test]
    fn test_build_sections_from_file_non_md() {
        let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        let result = build_sections_from_file(path);
        assert!(result.is_none(), "Expected None for a non-.md file");
    }

    #[test]
    fn record_page_sections_come_from_state_headings() {
        let tmp = TempDir::new().unwrap();
        let path = write_page(tmp.path(), "choose-db.md", RECORD_PAGE);

        let sections = build_sections_from_file(&path).expect("record page must yield sections");
        let headers: Vec<&str> = sections.iter().map(|s| s.header.as_str()).collect();
        assert_eq!(headers, vec!["Context", "Rationale"]);
        assert!(
            sections
                .iter()
                .all(|s| !s.body.contains("schema_version") && !s.body.contains("questions:")),
            "raw YAML keys must not leak into section bodies"
        );
    }

    #[test]
    fn record_page_inline_tags_come_from_state() {
        let tmp = TempDir::new().unwrap();
        let path = write_page(tmp.path(), "choose-db.md", RECORD_PAGE);

        let sections = build_sections_from_file(&path).expect("record page must yield sections");
        assert!(
            sections.iter().all(|s| s.tags.contains(&"storage".to_owned())),
            "inline tag inside state must be extracted"
        );
    }

    #[test]
    fn prose_page_sections_unchanged() {
        let tmp = TempDir::new().unwrap();
        let path = write_page(
            tmp.path(),
            "prose.md",
            "---\ntitle: Prose\ntype: concept\n---\n\n## Overview\n\nPlain prose body.\n",
        );

        let sections = build_sections_from_file(&path).expect("prose page must yield sections");
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header, "Overview");
        assert!(sections[0].body.contains("Plain prose body."));
    }

    fn build_index(sections: &[SectionDoc]) -> crate::search::Bm25Index {
        let docs = sections
            .iter()
            .map(crate::search::indexed_doc_from_section)
            .collect();
        crate::search::Bm25Index::build(docs)
    }

    #[test]
    fn record_state_term_still_ranks_the_page() {
        let tmp = TempDir::new().unwrap();
        let record_path = write_page(tmp.path(), "record.md", RECORD_PAGE);
        let prose_path = write_page(
            tmp.path(),
            "prose.md",
            "---\ntitle: Prose\ntype: concept\n---\n\n## Overview\n\nUnrelated prose.\n",
        );

        let mut sections = build_sections_from_file(&record_path).unwrap();
        let record_page_id = sections[0].page_id.clone();
        sections.extend(build_sections_from_file(&prose_path).unwrap());
        let index = build_index(&sections);

        let results = index.search("sqlite", 10);
        assert!(!results.is_empty(), "state term must be searchable");
        assert!(
            results[0].id.starts_with(&record_page_id),
            "record page must rank for a term unique to its state, got {:?}",
            results[0].id
        );
    }

    #[test]
    fn record_yaml_keys_do_not_pollute_search() {
        let tmp = TempDir::new().unwrap();
        let record_path = write_page(tmp.path(), "record.md", RECORD_PAGE);
        let sections = build_sections_from_file(&record_path).unwrap();
        let index = build_index(&sections);

        for token in ["schema_version", "questions", "instructions"] {
            let results = index.search(token, 10);
            assert!(
                results.is_empty(),
                "raw YAML token '{token}' must not match any section, got {:?}",
                results.iter().map(|r| &r.id).collect::<Vec<_>>()
            );
        }
    }

    const STRUCTURED_SPEC_PAGE: &str = "---\ntitle: Spec With FR\ntype: spec\nfunctional_requirements:\n  - {id: FR-9, description: \"Requirement phrase zzfrphrasezz for indexing\"}\nnon_functional_requirements:\n  - {id: NFR-2, description: \"Performance phrase zznfrphrasezz\"}\naliases: [zzaliaszz]\n---\n\n## Overview\n\nSpec body prose.\n";

    #[test]
    fn structured_frontmatter_values_are_indexed_but_keys_are_not() {
        let tmp = TempDir::new().unwrap();
        let path = write_page(tmp.path(), "spec.md", STRUCTURED_SPEC_PAGE);
        let sections = build_sections_from_file(&path).expect("spec page must yield sections");
        let index = build_index(&sections);

        for token in ["zzfrphrasezz", "zznfrphrasezz", "zzaliaszz"] {
            assert!(
                !index.search(token, 10).is_empty(),
                "structured value '{token}' must be indexed"
            );
        }
        for key in [
            "functional_requirements",
            "non_functional_requirements",
            "aliases",
        ] {
            assert!(
                sections.iter().all(|section| !section.body.contains(key)),
                "raw frontmatter key '{key}' must not appear in any section body"
            );
        }
        assert!(
            index.search("aliases", 10).is_empty(),
            "a key-only term must not match"
        );
    }
}
