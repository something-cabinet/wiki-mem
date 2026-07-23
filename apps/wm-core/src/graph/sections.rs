use std::path::Path;
use rayon::prelude::*;

use crate::engine::SectionDoc;
use crate::parser::{extract_frontmatter, extract_inline_tags, path_to_id, split_sections};

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
        .filter_map(|path| {
            let content = std::fs::read_to_string(path).ok()?;
            let rel_path = path.strip_prefix(wiki_dir).unwrap_or(path);
            let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
            let page_id = path_to_id(&rel_path_str);
            let (fm, body) = extract_frontmatter(&content);
            let title = fm.as_ref().and_then(|f| f.title.clone()).unwrap_or_else(|| {
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
            let section_docs: Vec<SectionDoc> = split_sections(body)
                .into_iter()
                .map(|(header, body_text)| {
                    let section_id =
                        format!("{}#{}", page_id, header.to_lowercase().replace(' ', "-"));
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
            Some(section_docs)
        })
        .flatten()
        .collect()
}
