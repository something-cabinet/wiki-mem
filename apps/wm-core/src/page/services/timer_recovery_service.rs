use std::sync::Arc;

use crate::engine::{EngineState, PageType};
use crate::error::ToolResult;
use crate::page_repo::{FsPageRepo, PageRepo};
use crate::parser;

use crate::page::helpers::page_path_helper::resolve_simple_page_path;

pub fn recover_orphan_timers_with_repo(
    engine: &Arc<EngineState>,
    repo: &dyn PageRepo,
) -> ToolResult<usize> {
    use chrono::Utc;
    let mut recovered: usize = 0;
    let snapshot = engine.graph.load();
    let graph = &snapshot.0;
    let index = &snapshot.1;

    for (page_id, node_idx) in index {
        let meta = &graph[*node_idx];
        if meta.page_type != PageType::Task {
            continue;
        }

        let Ok(path) = resolve_simple_page_path(page_id) else {
            continue;
        };
        let content = match repo.read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body) = parser::extract_frontmatter(&content);
        let fm = match fm {
            Some(f) => f,
            None => continue,
        };

        let time_started = match fm.time_started {
            Some(ref t) => t.clone(),
            None => continue,
        };

        let started_at = match chrono::DateTime::parse_from_rfc3339(&time_started) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let elapsed = Utc::now().signed_duration_since(started_at);
        if elapsed.num_hours() < 24 {
            continue;
        }

        let hours = elapsed.num_hours();
        let minutes = elapsed.num_minutes() % 60;
        let time_spent = format!("{}h {}m", hours, minutes);

        let mut new_fm = crate::parser::frontmatter_to_yaml(&fm);
        new_fm.push_str("status: done\n");
        new_fm.push_str(&format!("time_started: {}\n", time_started));
        new_fm.push_str(&format!("time_spent: {}\n", time_spent));

        let full = format!("---\n{}---\n\n{}", new_fm, body);
        if engine
            .write_channel
            .write(path.clone(), full.into_bytes())
            .is_ok()
        {
            tracing::info!(
                "Recovered orphan timer: {} ({} elapsed)",
                page_id,
                time_spent
            );
            recovered = recovered.wrapping_add(1);

            engine.emit_audit(
                "page.recover",
                "auto-close",
                "ok",
                0,
                None,
                vec![page_id.clone()],
            );
        }
    }

    Ok(recovered)
}

pub fn recover_orphan_timers(engine: &Arc<EngineState>) -> ToolResult<usize> {
    recover_orphan_timers_with_repo(engine, &FsPageRepo)
}
