use petgraph::stable_graph::StableGraph;

use crate::engine::{GraphEdge, WikiPageMeta, WriteChannel};
use crate::page::helpers::{build_frontmatter, FrontmatterValue};

pub fn auto_fix_missing_frontmatter(
    graph: &StableGraph<WikiPageMeta, GraphEdge>,
    write_channel: &WriteChannel,
) -> u64 {
    let mut fixed = 0u64;
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        let file_path = &meta.path;
        if !file_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body) = crate::parser::extract_frontmatter(&content);
        let fm = match fm {
            Some(f) => f,
            None => continue,
        };

        let mut needs_update = false;
        let title = fm.title.clone().unwrap_or_else(|| {
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

        let page_type = match fm.page_type {
            Some(ref pt) => pt.to_string(),
            None => {
                let parent = file_path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                needs_update = true;
                crate::engine::PageType::from_dir_name(&parent)
                    .map(|pt| pt.as_str().to_string())
                    .unwrap_or_else(|| crate::engine::PageType::Concept.as_str().to_string())
            }
        };

        if !needs_update {
            continue;
        }

        let mut fields: Vec<(&'static str, FrontmatterValue)> = vec![
            ("title", FrontmatterValue::Scalar(title)),
            ("type", FrontmatterValue::Scalar(page_type)),
        ];
        if !fm.tags.is_empty() {
            fields.push(("tags", FrontmatterValue::List(fm.tags.clone())));
        }
        if let Some(ref s) = fm.status {
            fields.push(("status", FrontmatterValue::Scalar(s.to_string())));
        }

        let full = format!("---\n{}---\n\n{}", build_frontmatter(&fields), body);
        write_channel
            .write(file_path.clone(), full.into_bytes())
            .ok();
        fixed = fixed.wrapping_add(1);
    }
    fixed
}
