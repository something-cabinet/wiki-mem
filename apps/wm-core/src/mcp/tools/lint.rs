use crate::mcp::prelude::*;
use sha2::{Digest, Sha256};
use wm_constants::*;

#[derive(Deserialize, JsonSchema)]
struct WmLintCheckInput {}

#[derive(Deserialize, JsonSchema)]
struct WmLintFixInput {}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    let e = engine.clone();
    registry.register_typed(
        "wm_lint.check",
        "Check wiki for common issues",
        move |_input: WmLintCheckInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let index = &snapshot.1;
            let mut issues = Vec::new();
            let mut cycle_info = None;

            for idx in graph.node_indices() {
                let meta = &graph[idx];

                let has_inbound = graph.edges_directed(idx, petgraph::Direction::Incoming).count() > 0;
                if !has_inbound {
                    issues.push(serde_json::json!({
                        "type": "orphan",
                        "severity": "warning",
                        "id": meta.id,
                        "message": "No inbound edges — consider adding relationships"
                    }));
                }

                for (_edge_type, target) in &meta.relates_to {
                    let normalized_target = target.replace('/', ":");
                    let resolved = index.get(&normalized_target)
                        .or_else(|| index.get(target))
                        .or_else(|| {
                            crate::parser::resolve_link_target(target, graph)
                                .and_then(|id| index.get(&id))
                        });
                    if resolved.is_none() {
                        issues.push(serde_json::json!({
                            "type": "unresolved_target",
                            "severity": "warning",
                            "id": meta.id,
                            "message": format!(
                                "Unresolved relates_to target '{}' from '{}' (page: {})",
                                target, meta.path.display(), meta.id
                            )
                        }));
                    }
                }

                if meta.page_type == crate::engine::PageType::Task && meta.task_data.as_ref().map(|d| d.acceptance_criteria.is_empty()).unwrap_or(true) {
                    issues.push(serde_json::json!({
                        "type": "missing_acs",
                        "severity": "warning",
                        "id": meta.id,
                        "message": "Task has no acceptance criteria"
                    }));
                }

                if meta.page_type == crate::engine::PageType::Spec {
                    let is_draft = meta.status == crate::engine::PageStatus::Draft;
                    if is_draft {
                        issues.push(serde_json::json!({
                            "type": "spec_status",
                            "severity": "info",
                            "id": meta.id,
                            "message": "Spec status is draft — consider setting reviewed/approved"
                        }));
                    }
                }

                let is_md_file = meta.path.extension().and_then(|ext| ext.to_str()) == Some("md");
                if is_md_file && meta.path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&meta.path) {
                        let trimmed = content.trim();
                        if let Some(rest) = trimmed.strip_prefix("---") {
                            if let Some(end) = rest.find("\n---") {
                                let frontmatter = &rest[..end];
                                let has_id = frontmatter
                                    .lines()
                                    .any(|line| line.starts_with("id:") || line.starts_with("id :"));
                                if !has_id {
                                    issues.push(serde_json::json!({
                                        "type": "missing_id",
                                        "severity": "warning",
                                        "id": meta.id,
                                        "message": format!(
                                            "Page '{}' is missing id: in frontmatter — add id: {}",
                                            meta.title, meta.id
                                        ),
                                    }));
                                }
                            }
                        }
                    }
                }

                let stem = meta
                    .path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if !stem.is_empty() {
                    if let Ok(file_content) = std::fs::read_to_string(&meta.path) {
                        let health = crate::parser::inspect_frontmatter_health(&file_content, &stem);
                        if let Some(bad_id) = health.scientific_notation_id {
                            issues.push(serde_json::json!({
                                "type": "scientific_notation_id",
                                "severity": "error",
                                "id": meta.id,
                                "message": format!(
                                    "Frontmatter id '{}' looks like a scientific-notation number and will be corrupted on the next YAML round-trip — quote it: id: \"{}\"",
                                    bad_id, bad_id
                                ),
                            }));
                        }
                        if health.duplicate_blocks {
                            issues.push(serde_json::json!({
                                "type": "duplicate_frontmatter",
                                "severity": "error",
                                "id": meta.id,
                                "message": "File contains duplicate '---' frontmatter blocks — merge into a single block",
                            }));
                        }
                        if meta.page_type == crate::engine::PageType::Task {
                            if let Some(mismatch) = health.id_mismatch {
                                issues.push(serde_json::json!({
                                    "type": "frontmatter_id_mismatch",
                                    "severity": "warning",
                                    "id": meta.id,
                                    "message": mismatch,
                                }));
                            }
                        }
                    }
                }
            }

            let registry_lock = e.source_registry.read().map_err(|_| crate::error::ToolError::lock_poisoned("registry"))?;
            let mut stale_count = 0usize;
            for entry in registry_lock.values() {
                let is_stale = if entry.state == crate::engine::SourceState::Stale {
                    true
                } else if entry.stored_path.exists() {
                    let content = match std::fs::read(&entry.stored_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    let current_hash = Sha256::digest(&content);
                    let current_hex = hex::encode(current_hash);
                    current_hex != entry.content_hash
                } else {
                    false
                };

                if is_stale {
                    stale_count = stale_count.wrapping_add(1);
                    issues.push(serde_json::json!({
                        "type": "stale_source",
                        "severity": "warning",
                        "id": entry.id,
                        "message": format!("Source '{}' content hash changed — needs reprocessing", entry.id),
                    }));
                }
            }
            if stale_count > 0 {
                issues.push(serde_json::json!({
                    "type": "stale_sources_summary",
                    "severity": "info",
                    "id": "registry",
                    "message": format!("{} stale source(s) found. Run source.verify to check.", stale_count),
                }));
            }

            if petgraph::algo::is_cyclic_directed(graph) {
                cycle_info = Some("Cycle detected in graph — expected for mutual relates_to links. BFS uses visited tracking.");
                issues.push(serde_json::json!({
                    "type": "cycle",
                    "severity": "info",
                    "id": "graph",
                    "message": "Cycle detected in wiki graph (expected: mutual relates_to links)"
                }));
            }

            if let Ok(root) = e.project_root.read().as_deref().cloned() {
                let root_wm = root.join(WM_DIR);
                let mut rogue_count = 0u32;
                let walker = walkdir::WalkDir::new(&root)
                    .into_iter()
                    .filter_entry(|entry| {
                        let name = entry.file_name().to_string_lossy();
                        !(name == "target" || name == ".git" || entry.path() == root_wm)
                    });

                for entry in walker.filter_map(|e| e.ok()) {
                    if entry.file_type().is_dir() && entry.file_name() == ".wm" {
                        rogue_count = rogue_count.wrapping_add(1);
                        issues.push(serde_json::json!({
                            "type": "rogue_wm_dir",
                            "severity": "error",
                            "id": entry.path().to_string_lossy(),
                            "message": format!(
                                "Found .wm/ directory outside project root: {}. Only one .wm/ is allowed — at project root.",
                                entry.path().display()
                            ),
                        }));
                    }
                }

                if rogue_count > 0 {
                    issues.push(serde_json::json!({
                        "type": "rogue_wm_summary",
                        "severity": "info",
                        "id": "filesystem",
                        "message": format!("{} rogue .wm/ director(ies) found. Run `find . -name .wm -type d` to locate.", rogue_count),
                    }));
                }
            }

            Ok(serde_json::json!({
                "issues": issues,
                "total": issues.len(),
                "has_cycles": cycle_info.is_some(),
                "stale_sources": stale_count,
            }))
        },
    );

    let e = engine.clone();
    registry.register_typed(
        "wm_lint.fix",
        "Auto-fix common issues",
        move |_input: WmLintFixInput| {
            let snapshot = e.graph.load();
            let graph = &snapshot.0;
            let fixed = crate::graph::auto_fix_missing_frontmatter(graph, &e.write_channel);

            if fixed > 0 {
                let e2 = e.clone();
                e.index_scheduler.submit("page", move || {
                    let root = e2
                        .project_root
                        .read()
                        .map(|r| r.clone())
                        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
                    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
                    let sections = crate::graph::build_sections_from_wiki(&wiki_dir);
                    let docs: Vec<crate::search::IndexedDoc> = sections
                        .iter()
                        .map(crate::search::indexed_doc_from_section)
                        .collect();
                    e2.bm25_index
                        .store(Arc::new(crate::search::Bm25Index::build(docs)));
                });
            }

            Ok(serde_json::json!({
                "fixed": fixed,
                "message": format!("Fixed {} issue(s)", fixed),
            }))
        },
    );
}
