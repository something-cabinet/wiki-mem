
use serde::Serialize;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use wm_constants::*;

const MAX_ATTACKER_STRING: usize = 256;

pub const KIND_PATH_ESCAPE: &str = "path_escape";
pub const KIND_HIDDEN_PATH: &str = "hidden_path";
pub const KIND_DISALLOWED_TOOL: &str = "disallowed_tool";
pub const KIND_INVALID_MODEL: &str = "invalid_model";
pub const KIND_SOURCE_REJECTED: &str = "source_rejected";
pub const KIND_AUTH_FAILURE: &str = "auth_failure";

#[derive(Debug, Clone, Serialize)]
pub struct SecurityAuditEvent {
    pub timestamp: String,
    pub category: String,
    pub kind: String,
    pub tool: String,
    pub detail: String,
    pub path: String,
}

pub fn sanitize(s: &str) -> String {
    let cleaned: String = s.chars().filter(|c| !c.is_control()).collect();
    if cleaned.chars().count() <= MAX_ATTACKER_STRING {
        cleaned
    } else {
        let cut: String = cleaned.chars().take(MAX_ATTACKER_STRING).collect();
        format!("{cut}…[truncated]")
    }
}

pub fn audit_log_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(LOG_FILE)
}

pub fn derive_project_root(confine_root: &Path) -> PathBuf {
    let norm = crate::shared::helpers::path_confine_helper::normalize_lexically(confine_root);
    let mut before: Vec<Component> = Vec::new();
    for component in norm.components() {
        if matches!(component, Component::Normal(c) if c == OsStr::new(WM_DIR)) {
            return before.iter().collect();
        }
        before.push(component);
    }
    PathBuf::from(".")
}

fn append_line(project_root: &Path, line: &str) {
    let wm_dir = project_root.join(WM_DIR);
    if !wm_dir.is_dir() {
        return;
    }
    let path = wm_dir.join(LOG_FILE);
    use std::io::Write;
    let mut file = match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("failed to open audit log {}: {}", path.display(), e);
            return;
        }
    };
    if let Err(e) = writeln!(file, "{line}") {
        tracing::warn!("failed to write audit log {}: {}", path.display(), e);
    }
    let _ = file.flush();
}

pub fn write_security_audit(project_root: &Path, event: &SecurityAuditEvent) {
    let Ok(line) = serde_json::to_string(event) else {
        return;
    };
    append_line(project_root, &line);
}

pub fn write_tool_audit(project_root: &Path, event: &crate::engine::AuditEvent) {
    let value = serde_json::json!({
        "timestamp": event.timestamp,
        "category": "tool",
        "kind": "tool_call",
        "tool": sanitize(&event.tool_name),
        "action": sanitize(&event.action),
        "duration_ms": event.duration_ms,
        "result": sanitize(&event.result),
        "error_message": event.error_message.as_deref().map(sanitize),
        "entity_refs": event.entity_refs.iter().map(|s| sanitize(s)).collect::<Vec<_>>(),
    });
    let Ok(line) = serde_json::to_string(&value) else {
        return;
    };
    append_line(project_root, &line);
}

pub fn audit_auth_failure(project_root: &Path, detail: &str) {
    write_security_audit(
        project_root,
        &SecurityAuditEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            category: "security".into(),
            kind: KIND_AUTH_FAILURE.into(),
            tool: "http_auth".into(),
            detail: sanitize(detail),
            path: String::new(),
        },
    );
}

pub fn audit_confine_rejection(confine_root: &Path, candidate: &Path, kind: &str) {
    let project_root = derive_project_root(confine_root);
    write_security_audit(
        &project_root,
        &SecurityAuditEvent {
            timestamp: chrono::Utc::now().to_rfc3339(),
            category: "security".into(),
            kind: kind.into(),
            tool: "path_confine_helper".into(),
            detail: sanitize(&candidate.to_string_lossy()),
            path: sanitize(&candidate.to_string_lossy()),
        },
    );
}
