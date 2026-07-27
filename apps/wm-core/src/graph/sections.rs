use rayon::prelude::*;
use std::path::Path;
use tracing::warn;

use crate::engine::SectionDoc;
use crate::parser::{extract_frontmatter, extract_inline_tags, path_to_id, split_sections};

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
    let title = fm
        .as_ref()
        .and_then(|f| f.title.clone())
        .unwrap_or_else(|| {
            path.file_stem()
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

    let sections: Vec<SectionDoc> = split_sections(body)
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

    if sections.is_empty() {
        warn!(
            "No sections found in {} (empty or unparseable)",
            path.display()
        );
        return None;
    }

    Some(sections)
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
}
