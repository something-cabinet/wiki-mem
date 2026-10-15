use std::io::Read;
use std::path::{Path, PathBuf};

use crate::shared::models::PageAction;
use crate::shared::services::call_tool;
use crate::shared::traits::Command;
use wm_constants::{WIKI_DIR, WM_DIR};
use wm_core::config;
use wm_core::page::RecordMigrationPlan;

use super::page_renderer::{
    print_page_created, print_page_deleted, print_page_get, print_page_list, print_page_linked,
    print_page_unlinked, print_page_updated, print_record_plan,
};

const PAGE_TOOL: &str = "wm_page";
const ACTION_GET: &str = "get";
const ACTION_LIST: &str = "list";
const ACTION_CREATE: &str = "create";
const ACTION_DELETE: &str = "delete";
const ACTION_UPDATE: &str = "update";
const ACTION_LINK: &str = "link";
const ACTION_UNLINK: &str = "unlink";
const ID_SEPARATOR: &str = "#";
const WIKI_PREFIX: &str = "wiki/";
const PATH_SEPARATOR: &str = "/";
const DEFAULT_PAGE_TYPE: &str = "concept";
const DEFAULT_EDGE_TYPE: &str = "relates_to";
const STDIN_READ_ERROR: &str = "Failed to read stdin";
const STDIN_JSON_ERROR: &str = "Invalid JSON on stdin";

impl Command for PageAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_page_action(self).await
    }
}

async fn run_page_action(action: PageAction) -> Result<(), anyhow::Error> {
    match action {
        PageAction::Get { id, json } => page_get(&id, json).await,
        PageAction::List { json } => page_list(json).await,
        PageAction::Create {
            path,
            title,
            page_type,
            json,
        } => page_create(&path, &title, &page_type, json).await,
        PageAction::Delete { id, json } => page_delete(&id, json).await,
        PageAction::Update { id, json } => page_update(&id, json).await,
        PageAction::Link {
            id,
            target,
            edge_type,
            json,
        } => page_link(&id, &target, &edge_type, json).await,
        PageAction::Unlink { id, target, json } => page_unlink(&id, &target, json).await,
        PageAction::MigrateRecords {
            apply,
            only,
            limit,
            json,
        } => page_migrate_records(apply, only, limit, json),
    }
}

async fn page_get(id: &str, json: bool) -> Result<(), anyhow::Error> {
    let page_id = id.split(ID_SEPARATOR).next().unwrap_or(id).to_string();
    let args = serde_json::json!({ "action": ACTION_GET, "id": page_id });
    match call_tool(PAGE_TOOL, args).await {
        Ok(resp) => print_page_get(&resp, id, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn page_list(json: bool) -> Result<(), anyhow::Error> {
    let args = serde_json::json!({ "action": ACTION_LIST });
    match call_tool(PAGE_TOOL, args).await {
        Ok(resp) => print_page_list(&resp, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn page_create(
    path: &str,
    title: &str,
    page_type: &Option<String>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let resolved = resolve_page_type(path, page_type);
    let content = read_stdin()?;
    let args = build_create_args(path, title, &content, page_type);
    match call_tool(PAGE_TOOL, args).await {
        Ok(resp) => print_page_created(&resp, path, &resolved, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn resolve_page_type(path: &str, page_type: &Option<String>) -> String {
    if let Some(explicit) = page_type {
        return explicit.clone();
    }
    infer_page_type(path)
}

fn infer_page_type(path: &str) -> String {
    let first_segment = path
        .trim_start_matches(WIKI_PREFIX)
        .split(PATH_SEPARATOR)
        .next()
        .unwrap_or(DEFAULT_PAGE_TYPE);
    wm_core::engine::PageType::from_dir_name(first_segment)
        .map(|page_type| page_type.as_str().to_string())
        .unwrap_or_else(|| DEFAULT_PAGE_TYPE.to_string())
}

fn build_create_args(
    path: &str,
    title: &str,
    content: &str,
    page_type: &Option<String>,
) -> serde_json::Value {
    let mut args = serde_json::json!({
        "action": ACTION_CREATE,
        "path": path,
        "title": title,
        "content": content,
    });
    if let Some(page_type) = page_type {
        args["type"] = serde_json::json!(page_type);
    }
    args
}

fn read_stdin() -> Result<String, anyhow::Error> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| anyhow::anyhow!("{STDIN_READ_ERROR}: {error}"))?;
    Ok(input)
}

async fn page_delete(id: &str, json: bool) -> Result<(), anyhow::Error> {
    let args = serde_json::json!({ "action": ACTION_DELETE, "id": id });
    match call_tool(PAGE_TOOL, args).await {
        Ok(resp) => print_page_deleted(&resp, id, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

async fn page_update(id: &str, json: bool) -> Result<(), anyhow::Error> {
    let mut args = read_update_payload()?;
    attach_update_identity(&mut args, id);
    match call_tool(PAGE_TOOL, args).await {
        Ok(_) => print_page_updated(id, json),
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn read_update_payload() -> Result<serde_json::Value, anyhow::Error> {
    let input = read_stdin()?;
    parse_update_payload(&input)
}

fn parse_update_payload(input: &str) -> Result<serde_json::Value, anyhow::Error> {
    serde_json::from_str(input).map_err(|error| anyhow::anyhow!("{STDIN_JSON_ERROR}: {error}"))
}

fn attach_update_identity(args: &mut serde_json::Value, id: &str) {
    let Some(object) = args.as_object_mut() else {
        return;
    };
    object.insert("action".into(), serde_json::json!(ACTION_UPDATE));
    object.insert("id".into(), serde_json::json!(id));
}

async fn page_link(
    id: &str,
    target: &str,
    edge_type: &Option<String>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let edge = edge_type.as_deref().unwrap_or(DEFAULT_EDGE_TYPE).to_string();
    let args = build_link_args(id, target, &edge);
    match call_tool(PAGE_TOOL, args).await {
        Ok(_) => print_page_linked(id, target, &edge, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn build_link_args(id: &str, target: &str, edge: &str) -> serde_json::Value {
    serde_json::json!({
        "action": ACTION_LINK,
        "id": id,
        "target": target,
        "edge_type": edge,
    })
}

async fn page_unlink(id: &str, target: &str, json: bool) -> Result<(), anyhow::Error> {
    let args = serde_json::json!({ "action": ACTION_UNLINK, "id": id, "target": target });
    match call_tool(PAGE_TOOL, args).await {
        Ok(_) => print_page_unlinked(id, target, json)?,
        Err(error) => eprintln!("Error: {}", error),
    }
    Ok(())
}

fn page_migrate_records(
    apply: bool,
    only: Vec<String>,
    limit: Option<usize>,
    json: bool,
) -> Result<(), anyhow::Error> {
    let wiki_dir = wiki_records_dir();
    let filter = wm_core::page::RecordMigrationFilter::new(only, limit);
    let plan = wm_core::page::plan_record_migration_filtered(&wiki_dir, &filter);
    print_record_plan(&plan, json)?;
    if apply {
        apply_record_migration(&wiki_dir, &plan);
    }
    Ok(())
}

fn wiki_records_dir() -> PathBuf {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    root.join(WM_DIR).join(WIKI_DIR)
}

fn apply_record_migration(wiki_dir: &Path, plan: &RecordMigrationPlan) {
    match wm_core::page::apply_wiki_record_migration(wiki_dir, plan) {
        Ok(written) => println!("Applied migration to {} page(s).", written),
        Err(error) => eprintln!("Migration apply failed: {}", error),
    }
}
