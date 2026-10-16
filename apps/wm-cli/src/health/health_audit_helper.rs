use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use petgraph::stable_graph::StableGraph;
use petgraph::visit::EdgeRef;
use wm_core::engine::{EdgeType, GraphEdge, SectionDoc, WikiPageMeta};
use wm_engine::parser::extract_frontmatter;

use crate::constants::fs::{ID_SEPARATOR, PATH_SEPARATOR};
use crate::constants::health::DEFAULT_EDGE_TYPE;
use crate::constants::wiki::{
    PAGE_ID_PREFIX, TASK_PREFIX, WIKI_FILE_EXT, WIKI_FILE_EXT_DOT, WIKI_INDEX_PAGE, WIKI_README_PAGE,
};

const SECTION_ID_SEPARATOR: char = '#';
const ACTIVE_STATUSES: [&str; 3] = ["todo", "in-progress", "draft"];

pub type Graph = StableGraph<WikiPageMeta, GraphEdge>;
pub type IdIndex = HashMap<String, petgraph::stable_graph::NodeIndex>;

pub fn collect_section_page_ids(sections: &[SectionDoc]) -> HashSet<String> {
    let mut page_ids = HashSet::new();
    for section in sections {
        let page_id = section
            .section_id
            .split(SECTION_ID_SEPARATOR)
            .next()
            .unwrap_or(&section.section_id)
            .to_string();
        page_ids.insert(page_id);
    }
    page_ids
}

pub fn scan_empty_pages(
    wiki_dir: &Path,
    section_page_ids: &HashSet<String>,
    graph: &Graph,
) -> Vec<serde_json::Value> {
    if !wiki_dir.exists() {
        return Vec::new();
    }
    let mut empty_pages = Vec::new();
    walk_wiki_dirs(wiki_dir, section_page_ids, graph, &mut empty_pages);
    empty_pages
}

fn walk_wiki_dirs(
    wiki_dir: &Path,
    section_page_ids: &HashSet<String>,
    graph: &Graph,
    empty_pages: &mut Vec<serde_json::Value>,
) {
    let mut dirs_to_visit = vec![wiki_dir.to_path_buf()];
    while let Some(dir) = dirs_to_visit.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(|entry| entry.ok()) {
            visit_wiki_entry(&entry.path(), wiki_dir, section_page_ids, graph, empty_pages, &mut dirs_to_visit);
        }
    }
}

fn visit_wiki_entry(
    path: &Path,
    wiki_dir: &Path,
    section_page_ids: &HashSet<String>,
    graph: &Graph,
    empty_pages: &mut Vec<serde_json::Value>,
    dirs_to_visit: &mut Vec<PathBuf>,
) {
    if path.is_dir() {
        dirs_to_visit.push(path.to_path_buf());
        return;
    }
    if is_wiki_page(path) {
        push_empty_page(path, wiki_dir, section_page_ids, graph, empty_pages);
    }
}

fn is_wiki_page(path: &Path) -> bool {
    path.extension().map(|ext| ext == WIKI_FILE_EXT).unwrap_or(false)
}

fn push_empty_page(
    path: &Path,
    wiki_dir: &Path,
    section_page_ids: &HashSet<String>,
    graph: &Graph,
    empty_pages: &mut Vec<serde_json::Value>,
) {
    let rel_path = relative_page_path(path, wiki_dir);
    let page_id = format!("{}{}", PAGE_ID_PREFIX, rel_path);
    if is_skipped_page(&page_id, section_page_ids) {
        return;
    }
    empty_pages.push(empty_page_json(&page_id, &rel_path, graph));
}

fn is_skipped_page(page_id: &str, section_page_ids: &HashSet<String>) -> bool {
    page_id == WIKI_INDEX_PAGE || page_id == WIKI_README_PAGE || section_page_ids.contains(page_id)
}

fn empty_page_json(page_id: &str, rel_path: &str, graph: &Graph) -> serde_json::Value {
    let inbound_refs = count_inbound_refs(graph, page_id);
    serde_json::json!({
        "id": page_id,
        "is_task": rel_path.starts_with(TASK_PREFIX),
        "inbound_refs": inbound_refs,
    })
}

fn relative_page_path(path: &Path, wiki_dir: &Path) -> String {
    path.strip_prefix(wiki_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(WIKI_FILE_EXT_DOT, "")
        .replace(PATH_SEPARATOR, &ID_SEPARATOR.to_string())
}

fn count_inbound_refs(graph: &Graph, page_id: &str) -> usize {
    graph
        .node_indices()
        .filter(|&idx| {
            graph
                .edges_directed(idx, petgraph::Direction::Outgoing)
                .any(|edge| graph[edge.target()].id == page_id)
        })
        .count()
}

pub fn scan_broken_refs(graph: &Graph, index: &IdIndex) -> Vec<serde_json::Value> {
    let mut broken_refs = Vec::new();
    for idx in graph.node_indices() {
        let meta = &graph[idx];
        for (edge_type, target) in &meta.relates_to {
            if !index.contains_key(target.as_str()) {
                broken_refs.push(broken_ref_json(&meta.id, target, edge_type));
            }
        }
    }
    broken_refs
}

fn broken_ref_json(source: &str, target: &str, edge_type: &EdgeType) -> serde_json::Value {
    serde_json::json!({ "source": source, "target": target, "edge_type": edge_type.as_yaml_str() })
}

pub fn case_insensitive_index(graph: &Graph) -> HashMap<String, String> {
    let mut ci_index = HashMap::new();
    for idx in graph.node_indices() {
        ci_index.insert(graph[idx].id.to_lowercase(), graph[idx].id.clone());
    }
    ci_index
}

pub fn group_broken_refs(
    broken_refs: &[serde_json::Value],
    ci_index: &HashMap<String, String>,
) -> BTreeMap<String, Vec<(String, String, bool)>> {
    let mut by_source: BTreeMap<String, Vec<(String, String, bool)>> = BTreeMap::new();
    for entry in broken_refs {
        let source = entry["source"].as_str().unwrap_or("").to_string();
        let target = entry["target"].as_str().unwrap_or("").to_string();
        let edge_type = entry["edge_type"].as_str().unwrap_or(DEFAULT_EDGE_TYPE).to_string();
        let is_case_fix = ci_index.contains_key(&target.to_lowercase());
        by_source.entry(source).or_default().push((target, edge_type, is_case_fix));
    }
    by_source
}

pub fn rebuild_refs(
    relates_to: &[(EdgeType, String)],
    refs: &[(String, String, bool)],
    ci_index: &HashMap<String, String>,
) -> (Vec<serde_json::Value>, u32, u32) {
    let mut new_refs = Vec::new();
    let mut fixed = 0u32;
    let mut removed = 0u32;
    for (edge_type, target) in relates_to {
        match refs.iter().find(|(candidate, _, _)| candidate == target) {
            Some((_, _, true)) => fixed += push_case_fix(&mut new_refs, edge_type, target, ci_index),
            Some(_) => removed += 1,
            None => new_refs.push(ref_json(edge_type, target)),
        }
    }
    (new_refs, fixed, removed)
}

fn push_case_fix(
    new_refs: &mut Vec<serde_json::Value>,
    edge_type: &EdgeType,
    target: &str,
    ci_index: &HashMap<String, String>,
) -> u32 {
    let Some(correct) = ci_index.get(&target.to_lowercase()) else {
        return 0;
    };
    new_refs.push(serde_json::json!({ "type": edge_type.as_yaml_str(), "target": correct }));
    1
}

fn ref_json(edge_type: &EdgeType, target: &str) -> serde_json::Value {
    serde_json::json!({ "type": edge_type.as_yaml_str(), "target": target })
}

pub fn stub_title(wiki_dir: &Path, page_id: &str) -> Option<String> {
    let path = page_markdown_path(wiki_dir, page_id);
    let content = read_nonempty_file(&path)?;
    let (frontmatter, body) = extract_frontmatter(&content);
    if !body.trim().is_empty() || !is_active_frontmatter(frontmatter.as_ref()) {
        return None;
    }
    Some(frontmatter_title(frontmatter.as_ref(), page_id))
}

pub fn page_markdown_path(wiki_dir: &Path, page_id: &str) -> PathBuf {
    let relative = page_id
        .strip_prefix(PAGE_ID_PREFIX)
        .unwrap_or(page_id)
        .replace(ID_SEPARATOR, &PATH_SEPARATOR.to_string());
    wiki_dir.join(format!("{}{}", relative, WIKI_FILE_EXT_DOT))
}

fn read_nonempty_file(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content)
}

fn is_active_frontmatter(frontmatter: Option<&WikiPageMeta>) -> bool {
    frontmatter
        .map(|fm| ACTIVE_STATUSES.contains(&fm.status.as_str()))
        .unwrap_or(true)
}

fn frontmatter_title(frontmatter: Option<&WikiPageMeta>, fallback: &str) -> String {
    frontmatter
        .map(|fm| fm.title.as_str())
        .filter(|title| !title.is_empty())
        .unwrap_or(fallback)
        .to_string()
}
