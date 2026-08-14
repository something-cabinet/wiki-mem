//! `wm_doc` — thin alias over the `wm_page` implementation path.
//!
//! Keeps the `wm_doc` tool name and its action-based, path-shaped input
//! schema (list/get/create/update/delete) for backward compatibility, but
//! every action is translated to the equivalent `WmPageAction` and executed
//! by `page::handle_action` — the same services `wm_page` uses
//! (`page_crud_service`, the page update builder, graph-index + filesystem
//! reads). There is exactly ONE writer; the historical doc.rs writer (its
//! private `parse_frontmatter` and the byte-imitation markdown builder that
//! re-introduced the unquoted-tags bug) is gone.
//!
//! `wm_doc.create`/`wm_doc.update` persist `type`/`tags` exactly as
//! `wm_page` does by construction, since the alias routes to the page path.
//!
//! Deprecation note — observable OUTPUT now matches `wm_page`, not the
//! historical `wm_doc` shape (this is intended convergence toward retiring
//! `wm_doc`; see spec `retire-wm-doc` / task
//! `execute-retire-wm-doc-consolidation`, which sanction dropping the
//! byte-imitation layer). Concretely: `get` returns the page shape
//! (`id`/`content`/`sections`/…) rather than `path`/`body`/`frontmatter`;
//! `list` returns `{pages:[{id,title,type,status}]}` rather than
//! `{docs:[…]}` and filters by page-TYPE name (`spec`) via
//! `PageType::from_type_name`, not by directory name (`specs`); `create`/
//! `update`/`delete` return the page result (`id`/`path`/`type`). Re-adding
//! a shape-imitation layer here would violate the `no-compensating-layers`
//! rule and the retire-wm-doc design, so the alias exposes the page contract
//! directly.

use crate::mcp::prelude::*;
use crate::mcp::tools::page::{handle_action, WmPageAction};
use crate::parser::path_to_id;
use crate::shared::helpers::path_confine_helper::confine;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wm_constants::*;

fn wiki_docs_dir(root: &Path) -> PathBuf {
    root.join(WM_DIR).join(WIKI_DIR)
}

fn ensure_md_ext(path: &str) -> String {
    if path.ends_with(".md") {
        path.to_string()
    } else {
        format!("{}.md", path)
    }
}

/// Backward-compatible `wm_doc` input schema (action-tagged, snake_case).
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
enum WmDocAction {
    #[schemars(description = "List documents in the wiki (.wm/wiki/)")]
    List { r#type: Option<String> },
    #[schemars(description = "Read a doc from .wm/wiki/ by path")]
    Get { path: String },
    #[schemars(description = "Create a new doc in .wm/wiki/")]
    Create {
        path: String,
        title: String,
        content: Option<String>,
        r#type: Option<String>,
        tags: Option<Vec<String>>,
    },
    #[schemars(description = "Update an existing doc")]
    Update {
        path: String,
        title: Option<String>,
        content: Option<String>,
        r#type: Option<String>,
        tags: Option<Vec<String>>,
    },
    #[schemars(description = "Delete a doc")]
    Delete { path: String },
}

/// Preserve `wm_doc`'s historical path-confinement guarantee. The page path
/// confines on create, but update/delete/get resolve through the graph index
/// (or a raw disk fallback) and would surface a traversal attempt as a plain
/// "not found" — the alias checks the raw path first so escapes fail with the
/// same "Access denied"/"escapes" error and audit event they always did.
fn confine_doc_path(engine: &Arc<EngineState>, path: &str) -> Result<(), ToolError> {
    let root = engine
        .project_root
        .read()
        .map_err(|_| ToolError::lock_poisoned("project_root"))?
        .clone();
    confine(&wiki_docs_dir(&root), Path::new(&ensure_md_ext(path)))?;
    Ok(())
}

pub fn register(registry: &mut ToolRegistry, engine: Arc<EngineState>) {
    registry.register_typed(
        "wm_doc",
        "Doc CRUD operations: list, get, create, update, delete (alias of wm_page)",
        move |input: WmDocAction| {
            let page_action = to_page_action(&engine, input)?;
            handle_action(&engine, page_action)
        },
    );
}

/// Translate the `wm_doc` action surface onto the `wm_page` action surface.
/// Path-bearing actions keep the historical confinement check; `path` maps to
/// a `wiki:`-prefixed page id via `path_to_id` exactly as `wm_page.create`
/// does for its own `path` input.
fn to_page_action(
    engine: &Arc<EngineState>,
    input: WmDocAction,
) -> Result<WmPageAction, ToolError> {
    match input {
        WmDocAction::List { r#type } => Ok(WmPageAction::List { r#type }),
        WmDocAction::Get { path } => {
            confine_doc_path(engine, &path)?;
            Ok(WmPageAction::Get {
                id: path_to_id(&path),
            })
        }
        WmDocAction::Create {
            path,
            title,
            content,
            r#type,
            tags,
        } => {
            confine_doc_path(engine, &path)?;
            Ok(WmPageAction::Create {
                path,
                title,
                content,
                r#type,
                tags,
                status: None,
            })
        }
        WmDocAction::Update {
            path,
            title,
            content,
            r#type,
            tags,
        } => {
            confine_doc_path(engine, &path)?;
            Ok(WmPageAction::Update {
                id: path_to_id(&path),
                title,
                content,
                status: None,
                tags,
                r#type,
                relates_to: None,
                notes: None,
                append_notes: None,
                extra_frontmatter: None,
            })
        }
        WmDocAction::Delete { path } => {
            confine_doc_path(engine, &path)?;
            Ok(WmPageAction::Delete {
                id: path_to_id(&path),
            })
        }
    }
}
