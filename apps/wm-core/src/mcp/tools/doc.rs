
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
