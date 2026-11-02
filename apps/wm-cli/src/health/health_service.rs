use std::path::PathBuf;
use std::sync::Arc;

use wm_constants::{WIKI_DIR, WM_DIR};
use wm_core::config;
use wm_core::engine::MainEngine;

use crate::constants::health::{HEALTH_STATUS_CLEAN, HEALTH_STATUS_ISSUES};
use crate::health::HealthAction;
use crate::models::output_format::OutputFormat;
use crate::shared::services::rebuild_from_engine;
use crate::shared::traits::Command;

use super::health_audit_helper::{collect_section_page_ids, scan_broken_refs, scan_empty_pages};
use super::health_fix_service::{apply_audit_fixes, fail_on_unapplied_fixes, FixReport};

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
    print_audit(&empty_pages, &broken_refs, &report, is_dry_run, &format)?;
    fail_on_unapplied_fixes(&report.failures)
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
