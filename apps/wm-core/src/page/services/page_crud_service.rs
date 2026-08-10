use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use wm_constants::*;
use wm_search::Bm25Index;

use crate::engine::{EngineState, PageType, WikiPageContent};
use crate::error::{ToolError, ToolResult};
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::parser::parse_wiki_page;
use crate::search::indexed_doc_from_section;

use crate::page::helpers::page_path_helper::resolve_page_path;

/// Anchor a wiki-relative page path (e.g. `.wm/wiki/tasks/foo.md`, as stored
/// in `meta.path`) to the project root so file I/O never double-prefixes
/// `.wm/wiki` when the process CWD is inside `.wm/wiki/` itself. Absolute
/// paths pass through untouched.
pub fn anchored_page_path(engine: &EngineState, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    root.join(path)
}

/// Resolve the wiki directory for an engine (`project_root/.wm/wiki`).
pub fn wiki_dir_for(engine: &EngineState) -> PathBuf {
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    root.join(WM_DIR).join(WIKI_DIR)
}

/// Resolve a page's metadata from the in-memory graph index, falling back to
/// disk when the index is stale (page exists on disk but hasn't been indexed
/// yet, e.g. created externally or before an index rebuild). Mirrors the
/// disk-resolution behavior of `get_page` so valid pages are never reported
/// as "not found" by update/delete/task handlers.
pub fn resolve_page_meta(
    engine: &EngineState,
    id: &str,
    repo: &dyn PageRepo,
) -> ToolResult<crate::engine::WikiPageMeta> {
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    if let Some(node_idx) = index.get(id) {
        return Ok(snapshot.0[*node_idx].clone());
    }
    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let path_part = id.split('#').next().unwrap_or(id).replace(':', "/");
    let path_part = path_part.strip_prefix("wiki/").unwrap_or(&path_part);
    let file_path = root
        .join(WM_DIR)
        .join(WIKI_DIR)
        .join(format!("{}.md", path_part));
    if !repo.exists(&file_path) {
        return Err(crate::error::ToolError::not_found("page", id));
    }
    let content = repo.read_to_string(&file_path)?;
    Ok(parse_wiki_page(&file_path, &content))
}


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
        let sections = crate::parser::parse_sections(file_path, content);
        for section in &sections {
            let doc = indexed_doc_from_section(section);
            bm25.add_document(doc);
        }
    }

    engine.bm25_index.store(Arc::new(bm25));

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
            let prefix = format!("{}#", page_id);
            corpus.retain(|s| !s.section_id.starts_with(&prefix));
            let new_sections = crate::parser::parse_sections(file_path, content);
            corpus.extend(new_sections);
        }
        corpus
    });

    engine.stale_flag.store(false, Ordering::Release);
}

/// Incrementally update the embedding vector store after a page mutation.
/// Mirrors `update_bm25_for_page`: removes every section vector belonging to
/// `page_id` (both in-memory and persisted), then — unless this is a delete —
/// embeds the freshly parsed sections and upserts them. Only the affected
/// page's sections are touched (no full re-embed). No-ops on embed failures;
/// if no embedder is loaded the stale vectors are still removed.
pub fn update_vectors_for_page(
    engine: &EngineState,
    page_id: &str,
    sections: &[crate::engine::SectionDoc],
    is_delete: bool,
) {
    engine.vector_store.remove_sections_for_page(page_id);

    if is_delete || sections.is_empty() || !engine.embedder.is_loaded() {
        return;
    }

    let mut entries = HashMap::new();
    let mut hashes = HashMap::new();
    for section in sections {
        match engine.embedder.embed(&section.body) {
            Ok(vec) => {
                let hash: [u8; 32] = Sha256::digest(section.body.as_bytes()).into();
                entries.insert(section.section_id.clone(), vec.normalized());
                hashes.insert(section.section_id.clone(), hash);
            }
            Err(e) => {
                tracing::warn!(
                    "Embedding failed for section {} (page {}): {}",
                    section.section_id,
                    page_id,
                    e
                );
            }
        }
    }
    if !entries.is_empty() {
        engine.vector_store.upsert_sections(entries, hashes);
    }
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
    let full_path = anchored_page_path(engine, &full_path);

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

    // Refresh the in-memory graph snapshot synchronously so reads (get/list/
    // board/neighbors) reflect the write immediately. wm-server boots
    // EngineState::new without the file watcher that MainEngineFactory spawns,
    // so without this the snapshot stays stale until an explicit rebuild.
    let wiki_dir = wiki_dir_for(engine);
    crate::graph::handle_file_change(&wiki_dir, &full_path, engine);

    let meta = parse_wiki_page(&full_path, &full_content);
    update_bm25_for_page(engine, &meta.id, &full_content, &full_path, false);

    let sections = crate::parser::parse_sections(&full_path, &full_content);
    update_vectors_for_page(engine, &meta.id, &sections, false);

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

    let root = engine
        .project_root
        .read()
        .map(|r| r.clone())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let file_path = crate::page::helpers::page_path_helper::resolve_id_to_path(&root, id)?;
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
    let page_id = id.split('#').next().unwrap_or(id);
    let snapshot = engine.graph.load();
    let index = &snapshot.1;
    let node_idx = index
        .get(page_id)
        .ok_or_else(|| ToolError::not_found("page", id))?;
    let meta = &snapshot.0[*node_idx];
    let file_path = anchored_page_path(engine, &meta.path);

    repo.read_to_string(&file_path)
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
    let file_path = anchored_page_path(engine, &meta.path);

    if repo.exists(&file_path) {
        repo.remove_file(&file_path)?;
    }

    // Refresh the in-memory graph snapshot synchronously (see
    // create_page_with_repo) so the deleted page disappears from get/list/
    // board immediately instead of lingering until an index rebuild.
    let wiki_dir = wiki_dir_for(engine);
    crate::graph::handle_file_delete(&wiki_dir, &file_path, engine);

    update_bm25_for_page(engine, id, "", &file_path, true);
    update_vectors_for_page(engine, id, &[], true);

    Ok(())
}

pub fn delete_page(engine: &Arc<EngineState>, id: &str) -> ToolResult<()> {
    delete_page_with_repo(engine, id, &FsPageRepo)
}
