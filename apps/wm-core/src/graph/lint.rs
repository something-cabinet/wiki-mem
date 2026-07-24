use petgraph::stable_graph::StableGraph;

use crate::engine::{EdgeType, WriteChannel, WikiPageMeta};

pub fn auto_fix_missing_frontmatter(
    graph: &StableGraph<WikiPageMeta, EdgeType>,
    write_channel: &WriteChannel,
) -> u64 {
    let mut fixed = 0u64;
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        let file_path = &meta.path;
        if !file_path.exists() { continue; }

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body) = crate::parser::extract_frontmatter(&content);
        let fm = match fm {
            Some(f) => f,
            None => continue,
        };

        let mut new_fm = String::new();
        let title = fm.title.as_deref().unwrap_or(
            file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled")
        );
        new_fm.push_str(&format!("title: {}\n", title));

        let mut needs_update = false;
        if fm.page_type.is_none() {
            let parent = file_path.parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let inferred = match parent.as_str() {
                "tasks" => "task",
                "specs" => "spec",
                "concepts" => "concept",
                "patterns" => "pattern",
                "decisions" => "decision",
                "howto" => "howto",
                "reference" => "reference",
                "rules" => "rule",
                "core" => "core",
                _ => "concept",
            };
            new_fm.push_str(&format!("type: {}\n", inferred));
            needs_update = true;
        } else if let Some(ref pt) = fm.page_type {
            new_fm.push_str(&format!("type: {}\n", pt));
        }

        if !fm.tags.is_empty() { new_fm.push_str(&format!("tags: [{}]\n", fm.tags.join(", "))); }
        if let Some(ref s) = fm.status { new_fm.push_str(&format!("status: {}\n", s)); }

        if needs_update {
            let full = format!("---\n{}---\n\n{}", new_fm, body);
            write_channel.write(file_path.clone(), full.into_bytes()).ok();
            fixed += 1;
        }
    }
    fixed
}
