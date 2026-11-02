use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use petgraph::stable_graph::NodeIndex;
use wm_core::engine::MainEngine;

use super::health_audit_helper::{
    case_insensitive_index, group_broken_refs, page_markdown_path, rebuild_refs, stub_acceptance,
    stub_title, Graph, IdIndex,
};

#[derive(Default)]
pub(super) struct FixReport {
    pub(super) deleted_pages: Vec<String>,
    pub(super) stubbed_pages: Vec<String>,
    pub(super) fixed_refs: u32,
    pub(super) removed_refs: u32,
    pub(super) failures: Vec<String>,
}

pub(super) fn fail_on_unapplied_fixes(failures: &[String]) -> Result<(), anyhow::Error> {
    if failures.is_empty() {
        return Ok(());
    }
    anyhow::bail!(
        "{} fix(es) could not be applied:\n  {}",
        failures.len(),
        failures.join("\n  ")
    )
}

pub(super) fn apply_audit_fixes(
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
        match delete_orphan_task(wiki_dir, page) {
            Ok(Some(page_id)) => report.deleted_pages.push(page_id),
            Ok(None) => {}
            Err(failure) => report.failures.push(failure),
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
        match stub_empty_task(engine, wiki_dir, page) {
            Ok(Some(page_id)) => report.stubbed_pages.push(page_id),
            Ok(None) => {}
            Err(failure) => report.failures.push(failure),
        }
    }
}

fn delete_orphan_task(wiki_dir: &Path, page: &serde_json::Value) -> Result<Option<String>, String> {
    let page_id = page["id"].as_str().unwrap_or("");
    let is_task = page["is_task"].as_bool().unwrap_or(false);
    let inbound = page["inbound_refs"].as_u64().unwrap_or(0);
    if !is_task || inbound != 0 {
        return Ok(None);
    }
    let path = page_markdown_path(wiki_dir, page_id);
    if !path.exists() {
        return Ok(None);
    }
    std::fs::remove_file(&path)
        .map(|()| Some(page_id.to_string()))
        .map_err(|error| format!("Failed to delete {}: {}", page_id, error))
}

fn stub_empty_task(
    engine: &Arc<MainEngine>,
    wiki_dir: &Path,
    page: &serde_json::Value,
) -> Result<Option<String>, String> {
    let page_id = page["id"].as_str().unwrap_or("");
    let is_task = page["is_task"].as_bool().unwrap_or(false);
    let inbound = page["inbound_refs"].as_u64().unwrap_or(0);
    if !is_task || inbound == 0 {
        return Ok(None);
    }
    let Some(title) = stub_title(wiki_dir, page_id) else {
        return Ok(None);
    };
    let body = format!(
        "## Overview\n\n(Task stub — no description yet for \"{}\".)\n\nAcceptance criteria and implementation details live in the wiki task page.",
        title
    );
    update_source_body(engine, page_id, body).map(Some)
}

fn update_source_body(
    engine: &Arc<MainEngine>,
    page_id: &str,
    body: String,
) -> Result<String, String> {
    let params = wm_core::page::PageUpdateParams {
        content: Some(body),
        acceptance_criteria: Some(vec![stub_acceptance()]),
        ..Default::default()
    };
    wm_core::page::update_page(&engine.state, page_id, &params)
        .map(|_| page_id.to_string())
        .map_err(|error| format!("Failed to stub {}: {}", page_id, error))
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
        match rewrite_source_refs(engine, graph, source_index, source, refs, ci_index) {
            Ok((fixed, removed)) => {
                report.fixed_refs += fixed;
                report.removed_refs += removed;
            }
            Err(failure) => report.failures.push(failure),
        }
    }
}

fn rewrite_source_refs(
    engine: &Arc<MainEngine>,
    graph: &Graph,
    source_index: NodeIndex,
    source: &str,
    refs: &[(String, String, bool)],
    ci_index: &HashMap<String, String>,
) -> Result<(u32, u32), String> {
    let (new_refs, fixed, removed) = rebuild_refs(&graph[source_index].relates_to, refs, ci_index);
    update_source_refs(engine, source, new_refs)?;
    Ok((fixed, removed))
}

fn update_source_refs(
    engine: &Arc<MainEngine>,
    source: &str,
    new_refs: Vec<serde_json::Value>,
) -> Result<(), String> {
    let params = wm_core::page::PageUpdateParams {
        relates_to: Some(new_refs),
        ..Default::default()
    };
    wm_core::page::update_page(&engine.state, source, &params)
        .map(|_| ())
        .map_err(|error| format!("Failed to update refs for {}: {}", source, error))
}
