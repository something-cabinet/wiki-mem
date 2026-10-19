use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use petgraph::stable_graph::NodeIndex;
use wm_constants::{WIKI_DIR, WM_DIR};
use wm_core::config;
use wm_core::engine::MainEngine;

use crate::constants::health::{HEALTH_STATUS_CLEAN, HEALTH_STATUS_ISSUES};
use crate::health::HealthAction;
use crate::models::output_format::OutputFormat;
use crate::shared::services::rebuild_from_engine;
use crate::shared::traits::Command;

use super::health_audit_helper::{
    case_insensitive_index, collect_section_page_ids, group_broken_refs, page_markdown_path,
    rebuild_refs, scan_broken_refs, scan_empty_pages, stub_title, Graph, IdIndex,
};

#[derive(Default)]
struct FixReport {
    deleted_pages: Vec<String>,
    stubbed_pages: Vec<String>,
    fixed_refs: u32,
    removed_refs: u32,
}

impl Command for HealthAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_health_action(self)
    }
}

fn run_health_action(action: HealthAction) -> Result<(), anyhow::Error> {
    let HealthAction::Audit {
        dry_run,
        fix,
        format,
    } = action;
    let (engine, wiki_dir) = open_health_engine();
    let is_dry_run = dry_run || !fix;
    let snapshot = engine.state.graph.snapshot();
    let sections = engine.state.index.section_corpus();
    let section_page_ids = collect_section_page_ids(&sections);
    let empty_pages = scan_empty_pages(&wiki_dir, &section_page_ids, &snapshot.0);
    let broken_refs = scan_broken_refs(&snapshot.0, &snapshot.1);
    let report = apply_audit_fixes(
        &engine,
        &wiki_dir,
        &snapshot.0,
        &snapshot.1,
        &empty_pages,
        &broken_refs,
        is_dry_run,
    );
    print_audit(&empty_pages, &broken_refs, &report, is_dry_run, &format)
}

fn open_health_engine() -> (Arc<MainEngine>, PathBuf) {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    let wiki_dir = root.join(WM_DIR).join(WIKI_DIR);
    let engine = Arc::new(MainEngine::new());
    if wiki_dir.exists() {
        rebuild_from_engine(&engine, &wiki_dir);
    }
    (engine, wiki_dir)
}

fn health_status_label(empty_count: usize, broken_count: usize) -> &'static str {
    if empty_count == 0 && broken_count == 0 {
        return HEALTH_STATUS_CLEAN;
    }
    HEALTH_STATUS_ISSUES
}

fn apply_audit_fixes(
    engine: &Arc<MainEngine>,
    wiki_dir: &Path,
    graph: &Graph,
    index: &IdIndex,
    empty_pages: &[serde_json::Value],
    broken_refs: &[serde_json::Value],
    is_dry_run: bool,
) -> FixReport {
    if is_dry_run {
        return FixReport::default();
    }
    let mut report = FixReport::default();
    let ci_index = case_insensitive_index(graph);
    delete_orphan_tasks(wiki_dir, empty_pages, &mut report);
    stub_empty_tasks(engine, wiki_dir, empty_pages, &mut report);
    fix_broken_refs(engine, graph, index, broken_refs, &ci_index, &mut report);
    report
}

fn delete_orphan_tasks(wiki_dir: &Path, empty_pages: &[serde_json::Value], report: &mut FixReport) {
    for page in empty_pages {
        if let Some(page_id) = delete_orphan_task(wiki_dir, page) {
            report.deleted_pages.push(page_id);
        }
    }
}

fn stub_empty_tasks(
    engine: &Arc<MainEngine>,
    wiki_dir: &Path,
    empty_pages: &[serde_json::Value],
    report: &mut FixReport,
) {
    for page in empty_pages {
        if let Some(page_id) = stub_empty_task(engine, wiki_dir, page) {
            report.stubbed_pages.push(page_id);
        }
    }
}

fn delete_orphan_task(wiki_dir: &Path, page: &serde_json::Value) -> Option<String> {
    let page_id = page["id"].as_str().unwrap_or("");
    let is_task = page["is_task"].as_bool().unwrap_or(false);
    let inbound = page["inbound_refs"].as_u64().unwrap_or(0);
    if !is_task || inbound != 0 {
        return None;
    }
    let path = page_markdown_path(wiki_dir, page_id);
    if !path.exists() {
        return None;
    }
    match std::fs::remove_file(&path) {
        Ok(()) => Some(page_id.to_string()),
        Err(error) => {
            eprintln!("  Failed to delete {}: {}", page_id, error);
            None
        }
    }
}

fn stub_empty_task(
    engine: &Arc<MainEngine>,
    wiki_dir: &Path,
    page: &serde_json::Value,
) -> Option<String> {
    let page_id = page["id"].as_str().unwrap_or("");
    let is_task = page["is_task"].as_bool().unwrap_or(false);
    let inbound = page["inbound_refs"].as_u64().unwrap_or(0);
    if !is_task || inbound == 0 {
        return None;
    }
    let title = stub_title(wiki_dir, page_id)?;
    let body = format!(
        "## Overview\n\n(Task stub — no description yet for \"{}\".)\n\nAcceptance criteria and implementation details live in the wiki task page.",
        title
    );
    update_source_body(engine, page_id, body)
}

fn update_source_body(engine: &Arc<MainEngine>, page_id: &str, body: String) -> Option<String> {
    let params = wm_core::page::PageUpdateParams {
        content: Some(body),
        ..Default::default()
    };
    if let Err(error) = wm_core::page::update_page(&engine.state, page_id, &params) {
        eprintln!("  Failed to stub {}: {}", page_id, error);
        return None;
    }
    Some(page_id.to_string())
}

fn fix_broken_refs(
    engine: &Arc<MainEngine>,
    graph: &Graph,
    index: &IdIndex,
    broken_refs: &[serde_json::Value],
    ci_index: &HashMap<String, String>,
    report: &mut FixReport,
) {
    let by_source = group_broken_refs(broken_refs, ci_index);
    for (source, refs) in &by_source {
        let Some(&source_index) = index.get(source) else {
            continue;
        };
        let (fixed, removed) =
            rewrite_source_refs(engine, graph, source_index, source, refs, ci_index);
        report.fixed_refs += fixed;
        report.removed_refs += removed;
    }
}

fn rewrite_source_refs(
    engine: &Arc<MainEngine>,
    graph: &Graph,
    source_index: NodeIndex,
    source: &str,
    refs: &[(String, String, bool)],
    ci_index: &HashMap<String, String>,
) -> (u32, u32) {
    let (new_refs, fixed, removed) = rebuild_refs(&graph[source_index].relates_to, refs, ci_index);
    update_source_refs(engine, source, new_refs);
    (fixed, removed)
}

fn update_source_refs(engine: &Arc<MainEngine>, source: &str, new_refs: Vec<serde_json::Value>) {
    let params = wm_core::page::PageUpdateParams {
        relates_to: Some(new_refs),
        ..Default::default()
    };
    if let Err(error) = wm_core::page::update_page(&engine.state, source, &params) {
        eprintln!("  Failed to update refs for {}: {}", source, error);
    }
}

fn print_audit(
    empty_pages: &[serde_json::Value],
    broken_refs: &[serde_json::Value],
    report: &FixReport,
    is_dry_run: bool,
    format: &str,
) -> Result<(), anyhow::Error> {
    let status_label = health_status_label(empty_pages.len(), broken_refs.len());
    if matches!(OutputFormat::from_str(format), OutputFormat::Json) {
        return print_audit_json(empty_pages, broken_refs, report, is_dry_run);
    }
    print_audit_text(empty_pages, broken_refs, report, is_dry_run, status_label);
    Ok(())
}

fn print_audit_json(
    empty_pages: &[serde_json::Value],
    broken_refs: &[serde_json::Value],
    report: &FixReport,
    is_dry_run: bool,
) -> Result<(), anyhow::Error> {
    let status = health_status_label(empty_pages.len(), broken_refs.len());
    let result = serde_json::json!({
        "empty_pages": empty_pages, "empty_count": empty_pages.len(),
        "broken_refs": broken_refs, "broken_ref_count": broken_refs.len(),
        "dry_run": is_dry_run, "status": status, "fix_applied": !is_dry_run,
        "deleted_pages": report.deleted_pages, "stubbed_pages": report.stubbed_pages,
        "case_fixed_refs": report.fixed_refs, "removed_refs": report.removed_refs,
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn print_audit_text(
    empty_pages: &[serde_json::Value],
    broken_refs: &[serde_json::Value],
    report: &FixReport,
    is_dry_run: bool,
    status_label: &str,
) {
    println!("Health Audit");
    println!("============");
    println!("  Dry-run: {}", is_dry_run);
    print_empty_pages(empty_pages);
    print_broken_refs(broken_refs);
    print_fix_summary(report, is_dry_run);
    println!("  Status: {}", status_label);
}

fn print_empty_pages(empty_pages: &[serde_json::Value]) {
    let is_task_count = empty_pages
        .iter()
        .filter(|page| page["is_task"].as_bool().unwrap_or(false))
        .count();
    println!(
        "  Empty pages: {} ({} task pages)",
        empty_pages.len(),
        is_task_count
    );
    for page in empty_pages {
        println!("    {}  [inbound: {}]", page["id"], page["inbound_refs"]);
    }
}

fn print_broken_refs(broken_refs: &[serde_json::Value]) {
    println!("  Broken refs: {}", broken_refs.len());
    for entry in broken_refs {
        println!(
            "    {} -> {} ({})",
            entry["source"], entry["target"], entry["edge_type"]
        );
    }
}

fn print_fix_summary(report: &FixReport, is_dry_run: bool) {
    if is_dry_run {
        return;
    }
    println!(
        "  Fix actions: {} pages deleted, {} pages stubbed, {} refs removed, {} refs case-corrected",
        report.deleted_pages.len(),
        report.stubbed_pages.len(),
        report.removed_refs,
        report.fixed_refs
    );
}
