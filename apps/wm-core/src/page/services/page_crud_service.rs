use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use wm_search::Bm25Index;

use crate::engine::{EngineState, PageType, WikiPageContent};
use crate::error::{ToolError, ToolResult};
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::parser::parse_wiki_page;
use crate::search::indexed_doc_from_section;

use crate::page::helpers::page_path_helper::resolve_page_path;

/// Incrementally update the BM25 index after a page mutation.
/// Removes all sections belonging to `page_id`, then (if not a delete)
/// parses the content and adds new sections. Uses ArcSwap copy-on-write
/// to avoid blocking readers.
pub fn update_bm25_for_page(
    engine: &EngineState,
    page_id: &str,
    content: &str,
    file_path: &Path,
    is_delete: bool,
) {
    let mut bm25 = Bm25Index::clone(&*engine.bm25_index.load());

    // Remove all sections for this page (both update and delete paths)
    let prefix = format!("{}#", page_id);
    let to_remove: Vec<String> = bm25
        .docs
        .iter()
        .filter(|d| d.id.starts_with(&prefix))
        .map(|d| d.id.clone())
        .collect();
    for id in &to_remove {
        bm25.remove_document(id);
    }

    if !is_delete {
        // Parse new sections and add them to BM25
        let sections = crate::parser::parse_sections(file_path, content);
        for section in &sections {
            let doc = indexed_doc_from_section(section);
            bm25.add_document(doc);
        }
    }

    engine.bm25_index.store(Arc::new(bm25));

    // Incrementally update section_corpus for stats accuracy
    let page_section_ids: std::collections::HashSet<String> = if is_delete {
        let prefix = format!("{}#", page_id);
        engine
            .section_corpus
            .load()
            .iter()
            .filter(|s| s.section_id.starts_with(&prefix))
            .map(|s| s.section_id.clone())
            .collect()
    } else {
        std::collections::HashSet::new()
    };
    engine.section_corpus.rcu(|old| {
        let mut corpus = (**old).clone();
        if is_delete {
            corpus.retain(|s| !page_section_ids.contains(&s.section_id));
        } else {
            // Remove old sections for this page, then add new
            let prefix = format!("{}#", page_id);
            corpus.retain(|s| !s.section_id.starts_with(&prefix));
            let new_sections = crate::parser::parse_sections(file_path, content);
            corpus.extend(new_sections);
        }
        corpus
    });

    // Index is now up-to-date — clear stale flag
    engine.stale_flag.store(false, Ordering::Release);
}

pub fn create_page_with_repo(
    engine: &Arc<EngineState>,
    path: &str,
    frontmatter: &str,
    content: &str,
    repo: &dyn PageRepo,
) -> ToolResult<String> {
    let full_path = resolve_page_path(
        &engine
            .config
            .read()
            .map_err(|_| ToolError::lock_poisoned("config"))?
            .project_name,
        path,
    )?;

    let full_content = if frontmatter.trim().is_empty() {
        content.to_string()
    } else {
        format!("---\n{}---\n\n{}", frontmatter, content)
    };

    repo.create_dir_all(
        full_path
            .parent()
            .ok_or_else(|| ToolError::internal("invalid path"))?,
    )?;
    repo.write(&full_path, full_content.as_bytes())?;

    engine.notify_file_changed(&full_path);

    let meta = parse_wiki_page(&full_path, &full_content);
    update_bm25_for_page(engine, &meta.id, &full_content, &full_path, false);

    Ok(meta.id)
}

pub fn create_page(
    engine: &Arc<EngineState>,
    path: &str,
    frontmatter: &str,
    content: &str,
) -> ToolResult<String> {
    create_page_with_repo(engine, path, frontmatter, content, &FsPageRepo)
}

pub fn get_page_with_repo(
    engine: &Arc<EngineState>,
    id: &str,
    repo: &dyn PageRepo,
) -> ToolResult<WikiPageContent> {
    let snapshot = engine.graph.load();
    let id_index = &snapshot.1;

    let _node_idx = id_index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;

    let root = Path::new(".");
    let file_path = crate::page::helpers::page_path_helper::resolve_id_to_path(root, id)?;
    let content = repo.read_to_string(&file_path)?;

    let sections = crate::parser::parse_sections(&file_path, &content);

    Ok(WikiPageContent {
        raw: content,
        sections,
        meta: None,
    })
}

pub fn get_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<WikiPageContent> {
    get_page_with_repo(engine, id, &FsPageRepo)
}

/// Normalize a page ID by stripping any #section anchor suffix.
/// "wiki:reference:design-patterns#overview" → "wiki:reference:design-patterns"
pub fn normalize_page_id(id: &str) -> &str {
    id.split('#').next().unwrap_or(id)
}

pub fn get_page_raw_with_repo(
    engine: &EngineState,
    id: &str,
    repo: &dyn PageRepo,
) -> ToolResult<String> {
    // Strip #section anchor if present so "wiki:page#overview" resolves to "wiki:page"
    let page_id = id.split('#').next().unwrap_or(id);
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(page_id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    repo.read_to_string(file_path)
        .map_err(|e| ToolError::internal(format!("Failed to read {}: {}", file_path.display(), e)))
}

pub fn get_page_raw(engine: &EngineState, id: &str) -> ToolResult<String> {
    get_page_raw_with_repo(engine, id, &FsPageRepo)
}

pub fn list_pages(
    engine: &Arc<EngineState>,
    page_type_filter: Option<&PageType>,
) -> ToolResult<Vec<serde_json::Value>> {
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;

    let pages: Vec<serde_json::Value> = graph
        .node_indices()
        .filter_map(|idx| {
            let meta = &graph[idx];
            if let Some(pt) = page_type_filter {
                if meta.page_type != *pt {
                    return None;
                }
            }
            Some(serde_json::json!({
                "id": meta.id,
                "title": meta.title,
                "type": meta.page_type.as_str(),
                "status": meta.status.as_str(),
            }))
        })
        .collect();

    Ok(pages)
}

pub fn delete_page_with_repo(
    engine: &Arc<EngineState>,
    id: &str,
    repo: &dyn PageRepo,
) -> ToolResult<()> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = &meta.path;

    if repo.exists(file_path) {
        repo.remove_file(file_path)?;
    }

    update_bm25_for_page(engine, id, "", file_path, true);

    Ok(())
}

pub fn delete_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    delete_page_with_repo(engine, id, &FsPageRepo)
}
