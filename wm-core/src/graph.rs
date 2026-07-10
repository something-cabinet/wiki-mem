use arc_swap::ArcSwap;
use petgraph::algo::is_cyclic_directed;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

use crate::engine::{EdgeType, GraphSnapshot, SectionDoc, WikiPageMeta};
use crate::parser::{extract_frontmatter, parse_edge_type, parse_wiki_page, split_sections};

/// Scan wiki dir and build sections for BM25
pub fn build_sections_from_wiki(wiki_dir: &Path) -> Vec<SectionDoc> {
    // Collect file paths (sequential walkdir — fast)
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

    // Parallel read + parse into section groups, then flatten
    paths
        .par_iter()
        .filter_map(|path| {
            let content = std::fs::read_to_string(path).ok()?;
            let rel_path = path.strip_prefix(wiki_dir).unwrap_or(path);
            let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
            let page_id = crate::parser::path_to_id(&rel_path_str);
            let (_, body) = extract_frontmatter(&content);
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
                    }
                })
                .collect();
            Some(section_docs)
        })
        .flatten()
        .collect()
}

/// Scan a wiki directory and build a StableGraph + id_index
/// `registered_custom_types` — optional list of allowed custom edge type names.
/// Unregistered custom types are rejected with a warning.
pub fn build_graph_from_wiki(
    wiki_dir: &Path,
    registered_custom_types: &[String],
) -> (
    StableGraph<WikiPageMeta, EdgeType>,
    HashMap<String, petgraph::stable_graph::NodeIndex>,
) {
    let mut graph = StableGraph::<WikiPageMeta, EdgeType>::new();
    let mut id_index = HashMap::new();

    // Collect file paths (sequential walkdir — fast)
    let paths: Vec<_> = walkdir::WalkDir::new(wiki_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            name != "index.md" && name != "log.md"
        })
        .map(|e| (e.path().to_path_buf(), e.path().to_path_buf()))
        .collect();

    // Parallel: read + parse each file
    struct ParsedPage {
        meta: WikiPageMeta,
        edges: Vec<(String, String)>, // (edge_type_str, target)
        custom_types: Vec<String>,
    }

    let parsed: Vec<ParsedPage> = paths
        .par_iter()
        .filter_map(|(path, _)| {
            let rel_path = path.strip_prefix(wiki_dir).unwrap_or(path);
            let content = std::fs::read_to_string(path).ok()?;
            if content.trim().is_empty() {
                return None;
            }
            let meta = parse_wiki_page(rel_path, &content);
            let mut edges: Vec<(String, String)> = Vec::new();
            let mut custom_types: Vec<String> = Vec::new();
            for rel_str in &meta.relates_to {
                if let Some((edge_type_str, target)) = rel_str.split_once(':') {
                    edges.push((edge_type_str.to_string(), target.to_string()));
                    if is_custom_edge(edge_type_str)
                        && !custom_types.contains(&edge_type_str.to_string())
                    {
                        custom_types.push(edge_type_str.to_string());
                    }
                }
            }
            Some(ParsedPage { meta, edges, custom_types })
        })
        .collect();

    // Sequential: build graph + collect pending edges + track custom types
    let mut pending_edges: Vec<(String, String, String)> = Vec::new(); // (source_id, edge_type, target)
    let mut used_custom_types: Vec<String> = Vec::new();

    for page in parsed {
        let node_idx = graph.add_node(page.meta.clone());
        id_index.insert(page.meta.id.clone(), node_idx);
        for (edge_type_str, target) in page.edges {
            pending_edges.push((page.meta.id.clone(), edge_type_str, target));
        }
        for ct in page.custom_types {
            if !used_custom_types.contains(&ct) {
                used_custom_types.push(ct);
            }
        }
    }

    // Validate custom edge types against registered list
    let rejected = validate_custom_edge_types(registered_custom_types, &used_custom_types);
    for t in &rejected {
        warn!(
            "Custom edge type '{}' not registered in config. Skipping edges of this type.",
            t
        );
    }

    // In-memory edge addition pass (no disk I/O)
    let mut added_edges: std::collections::HashSet<(
        petgraph::stable_graph::NodeIndex,
        petgraph::stable_graph::NodeIndex,
    )> = std::collections::HashSet::new();
    for (source_id, edge_type_str, target) in &pending_edges {
        if rejected.contains(edge_type_str) {
            continue;
        }
        if let Some(&source_idx) = id_index.get(source_id) {
            // Try exact ID match first, then resolve-link for wikilinks (title/alias match)
            let target_idx = id_index.get(target).copied().or_else(|| {
                crate::parser::resolve_link_target(target, &graph)
                    .and_then(|id| id_index.get(&id).copied())
            });
            if let Some(target_idx) = target_idx {
                if let Ok(edge_type) = parse_edge_type(edge_type_str) {
                    // Deduplicate: skip if same (source, target) edge already added
                    if added_edges.insert((source_idx, target_idx)) {
                        graph.add_edge(source_idx, target_idx, edge_type);
                    }
                }
            }
        }
    }

    // Cycle detection (diagnostic only, never mutate)
    if is_cyclic_directed(&graph) {
        warn!("Cycle detected in wiki graph. BFS will use visited tracking to prevent infinite loops.");
    } else {
        info!("Graph is acyclic — safe for topological operations.");
    }

    (graph, id_index)
}

/// Rebuild the graph snapshot atomically via ArcSwap
pub fn rebuild_snapshot(
    graph_swap: &ArcSwap<GraphSnapshot>,
    wiki_dir: &Path,
    custom_types: &[String],
) -> usize {
    let (graph, id_index) = build_graph_from_wiki(wiki_dir, custom_types);
    let node_count = graph.node_count();
    let snapshot = Arc::new((graph, id_index));
    graph_swap.store(snapshot);
    info!("Graph rebuilt: {} nodes, replaced atomically.", node_count);
    node_count
}

/// Validate custom edge types against a registered set
pub fn validate_custom_edge_types(registered: &[String], used_types: &[String]) -> Vec<String> {
    let mut rejected = Vec::new();
    for t in used_types {
        if !registered.contains(t) && is_custom_edge(t) {
            rejected.push(t.clone());
        }
    }
    rejected
}

/// Auto-generate wiki/index.md from the current graph snapshot.
pub fn auto_generate_index(
    wiki_dir: &Path,
    graph: &StableGraph<WikiPageMeta, EdgeType>,
) -> Result<(), String> {
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str("title: Wiki Index\n");
    content.push_str("type: reference\n");
    content.push_str("---\n\n");
    content.push_str("# Wiki Index\n\n");
    content.push_str(
        "> **Do not edit this file manually.** It is auto-generated by `wm index rebuild`.\n\n",
    );

    // Group pages by type
    let mut by_type: std::collections::BTreeMap<String, Vec<&WikiPageMeta>> =
        std::collections::BTreeMap::new();
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        let type_name = format!("{:?}", meta.page_type).to_lowercase();
        by_type.entry(type_name).or_default().push(meta);
    }

    for (type_name, pages) in &by_type {
        content.push_str(&format!("## {}s\n\n", type_name));
        for meta in pages {
            let status = format!("{:?}", meta.status).to_lowercase();
            let link = format!("{}.md", meta.id.replace(':', "/"));
            content.push_str(&format!("- [{}]({}) — *{}*\n", meta.title, link, status));
        }
        content.push('\n');
    }

    // Graph stats
    content.push_str("## Graph Stats\n\n");
    content.push_str(&format!("- **Nodes:** {}\n", graph.node_count()));
    content.push_str(&format!("- **Edges:** {}\n", graph.edge_count()));
    content.push('\n');

    let index_path = wiki_dir.join("index.md");
    std::fs::write(&index_path, &content).map_err(|e| format!("write index.md: {}", e))?;
    Ok(())
}

/// Auto-fix common lint issues (missing title, missing type) on all graph pages.
/// Returns count of pages fixed.
pub fn lint_fix(
    graph: &StableGraph<WikiPageMeta, EdgeType>,
    write_channel: &crate::engine::WriteChannel,
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

fn is_custom_edge(s: &str) -> bool {
    matches!(parse_edge_type(s), Ok(EdgeType::Custom(_)))
}

/// Find shortest path between two nodes using BFS.
/// Returns list of (node_id, title, edge_type_from_parent).
pub fn find_path(
    graph: &petgraph::stable_graph::StableGraph<crate::engine::WikiPageMeta, crate::engine::EdgeType>,
    _index: &HashMap<String, petgraph::stable_graph::NodeIndex>,
    start: petgraph::stable_graph::NodeIndex,
    end: petgraph::stable_graph::NodeIndex,
    max_depth: usize,
) -> Vec<(String, String, Option<String>)> {
    use std::collections::{HashMap, HashSet, VecDeque};
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut parent: HashMap<
        petgraph::stable_graph::NodeIndex,
        (petgraph::stable_graph::NodeIndex, String),
    > = HashMap::new();
    visited.insert(start);
    queue.push_back((start, 0usize));
    let mut found = false;
    while let Some((current, depth)) = queue.pop_front() {
        if current == end {
            found = true;
            break;
        }
        if depth >= max_depth {
            continue;
        }
        for edge in graph.edges(current) {
            let target = edge.target();
            if visited.insert(target) {
                let edge_type = format!("{:?}", edge.weight()).to_lowercase();
                parent.insert(target, (current, edge_type));
                queue.push_back((target, depth + 1));
            }
        }
    }
    if !found {
        return vec![];
    }
    let mut path = Vec::new();
    let mut current = end;
    while current != start {
        if let Some((prev, edge_type)) = parent.get(&current) {
            path.push((
                graph[current].id.clone(),
                graph[current].title.clone(),
                Some(edge_type.clone()),
            ));
            current = *prev;
        } else {
            break;
        }
    }
    path.push((graph[start].id.clone(), graph[start].title.clone(), None));
    path.reverse();
    path
}
