//! Shared audit sink for security rejections.
//!
//! Security rejections (path escapes, hidden-path blocks, disallowed tools,
//! invalid model names, source-path rejections) are written as JSON lines to
//! `<project_root>/.wm/log.jsonl` — the same file `wm_log.*` reads — so a
//! prompt-injected agent probing for traversal leaves a durable, queryable
//! trace even when a transport layer discards the in-memory audit channel.
//!
//! Attacker-controlled strings are sanitized (control characters stripped) and
//! truncated before persistence so the audit log can never become a
//! log-injection or prompt-injection vector.

use serde::Serialize;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use wm_constants::*;

/// Maximum length of an attacker-controlled string before truncation.
const MAX_ATTACKER_STRING: usize = 256;

/// Event kinds recognised by the sink.
pub const KIND_PATH_ESCAPE: &str = "path_escape";
pub const KIND_HIDDEN_PATH: &str = "hidden_path";
pub const KIND_DISALLOWED_TOOL: &str = "disallowed_tool";
pub const KIND_INVALID_MODEL: &str = "invalid_model";
pub const KIND_SOURCE_REJECTED: &str = "source_rejected";
pub const KIND_AUTH_FAILURE: &str = "auth_failure";

/// A single audit log entry for a security rejection.
#[derive(Debug, Clone, Serialize)]
pub struct SecurityAuditEvent {
    pub timestamp: String,
    pub category: String,
    pub kind: String,
    pub tool: String,
    pub detail: String,
    pub path: String,
}

/// Strip control characters and truncate an attacker-controlled string.
pub fn sanitize(s: &str) -> String {
    let cleaned: String = s.chars().filter(|c| !c.is_control()).collect();
    if cleaned.chars().count() <= MAX_ATTACKER_STRING {
        cleaned
    } else {
        let cut: String = cleaned.chars().take(MAX_ATTACKER_STRING).collect();
        format!("{cut}…[truncated]")
    }
}

/// Path of the audit log for a project root.
pub fn audit_log_path(project_root: &Path) -> PathBuf {
    project_root.join(WM_DIR).join(LOG_FILE)
}

/// Best-effort derivation of the project root from a confinement `root`
/// argument.
///
/// Roots built from `project_root.join(WM_DIR).join(...)` contain a `.wm`
/// component; everything before it is the project root. Roots without a `.wm`
/// component (the raw project root passed by `wm_template`, or a `source_dirs`
/// base) fall back to the current directory — matching where `wm_log.*` reads.
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

/// Append a raw JSON line to `<project_root>/.wm/log.jsonl`.
///
/// Auditing is best-effort and never propagates errors into the caller. If no
/// `.wm` directory exists under the derived project root (e.g. a library
/// unit-test CWD), the event is dropped.
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

/// Append a JSON-line security audit event to `<project_root>/.wm/log.jsonl`.
pub fn write_security_audit(project_root: &Path, event: &SecurityAuditEvent) {
    let Ok(line) = serde_json::to_string(event) else {
        return;
    };
    append_line(project_root, &line);
}

/// Persist a general tool-audit event arriving on the in-memory audit channel.
///
/// `EngineState::emit_audit` records benign operational events (e.g. timer
/// recovery). The daemon drains the channel here instead of discarding so the
/// project's audit log keeps a durable record. Attacker-controlled fields are
/// sanitized and truncated with the same contract as the security events, and
/// serde_json escaping keeps a single event on a single physical line.
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

/// Emit an `auth_failure` event.
///
/// `detail` must describe the rejected request (route + method only) and must
/// NEVER include the credential that was rejected.
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

/// Emit a path-confinement rejection event (path escape or hidden path).
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
