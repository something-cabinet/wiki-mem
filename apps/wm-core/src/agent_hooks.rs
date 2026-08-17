//! Agent hook / instruction config for the "query-before-grep" rule.
//!
//! Pure, testable helpers that produce the instruction text and the
//! per-platform hook/enforcement config emitted by `wm setup <platform>`.
//! Nothing here touches the MCP tool surface — these are agent-side
//! instruction + permission config files only.

use crate::embed_files::EmbeddedFiles;
use serde_json::Value;

/// Relative path (from the project root) where the shared instruction file
/// is written by `wm setup`.
pub const QUERY_BEFORE_GREP_REL_PATH: &str = ".wm/agent/query-before-grep.md";

/// The instruction file reference injected into platform configs.
pub const QUERY_BEFORE_GREP_REF: &str = "./.wm/agent/query-before-grep.md";

/// Marker line present only in the strict variant (used by tests to tell the
/// two files apart).
pub const STRICT_MARKER: &str = "## Strict mode — enforced";

/// Markdown content of the query-before-grep instruction file.
pub fn query_before_grep_content(strict: bool) -> String {
    let key = if strict {
        "agent_instructions/query-before-grep-strict.md"
    } else {
        "agent_instructions/query-before-grep.md"
    };
    EmbeddedFiles::get(key)
        .and_then(|f| std::str::from_utf8(f.data.as_ref()).ok().map(String::from))
        .unwrap_or_else(|| panic!("embedded instruction template not found: {key}"))
}

/// True when the platform has a permission-rule mechanism that can gate raw
/// file reads (not merely instruction files).
pub fn platform_supports_enforcement(platform: &str) -> bool {
    matches!(platform, "opencode" | "claude")
}

/// opencode (>= 1.18) permission block: gate the raw-file-read tool surfaces
/// with `ask`. opencode has no session-state predicate ("only after a wm tool
/// ran"), so the enforcement is an explicit-approval gate on reads; the
/// instruction file tells the agent to query `wm_graph`/`wm_search` first.
pub fn opencode_strict_permission() -> Value {
    serde_json::json!({
        "read": "ask",
        "grep": "ask",
        "bash": "ask"
    })
}

/// Patch an opencode.json value: reference the query-before-grep instruction
/// file (always) and, when `strict`, add the read-gating permission block.
pub fn opencode_with_hook(mut cfg: Value, strict: bool) -> Value {
    if let Some(instructions) = cfg
        .get_mut("instructions")
        .and_then(|v| v.as_array_mut())
    {
        if !instructions
            .iter()
            .any(|v| v.as_str() == Some(QUERY_BEFORE_GREP_REF))
        {
            instructions.push(Value::String(QUERY_BEFORE_GREP_REF.to_string()));
        }
    }
    if strict {
        let strict_perm = opencode_strict_permission();
        let strict_perm = strict_perm.as_object().expect("static json object");
        match cfg.get_mut("permission").and_then(|v| v.as_object_mut()) {
            Some(existing) => {
                for (k, v) in strict_perm {
                    existing.insert(k.clone(), v.clone());
                }
            }
            None => {
                cfg["permission"] = Value::Object(strict_perm.clone());
            }
        }
    }
    cfg
}

/// Claude Code `.claude/settings.json` content for strict mode: ask before any
/// project file read (`Read(//**)`) or shell command (`Bash(*)`), so the first
/// raw read requires explicit approval.
pub fn claude_strict_settings() -> Value {
    serde_json::json!({
        "permissions": {
            "ask": ["Read(//**)", "Bash(*)"]
        }
    })
}

/// Import line appended to CLAUDE.md / AGENTS.md instruction files so the
/// query-before-grep guidance is pulled into platforms that read those files.
pub fn instruction_import_line() -> String {
    format!("@{}", QUERY_BEFORE_GREP_REL_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_differs_by_strictness() {
        let base = query_before_grep_content(false);
        let strict = query_before_grep_content(true);
        assert!(base.contains("wm_graph"));
        assert!(base.contains("wm_search"));
        assert!(!base.contains(STRICT_MARKER));
        assert!(strict.contains(STRICT_MARKER));
    }

    #[test]
    fn opencode_hook_adds_instruction_ref() {
        let cfg = serde_json::json!({
            "instructions": ["./OPENCODE.md"],
            "mcp": { "wm": { "command": ["wm-cli", "mcp"], "enabled": true } }
        });
        let patched = opencode_with_hook(cfg, false);
        let instructions = patched["instructions"].as_array().unwrap();
        assert!(instructions
            .iter()
            .any(|v| v.as_str() == Some(QUERY_BEFORE_GREP_REF)));
        assert!(patched.get("permission").is_none());
    }

    #[test]
    fn opencode_strict_adds_permission_gate() {
        let cfg = serde_json::json!({ "instructions": [] });
        let patched = opencode_with_hook(cfg, true);
        let permission = patched["permission"].as_object().unwrap();
        for tool in ["read", "grep", "bash"] {
            assert_eq!(permission[tool], "ask", "strict should gate {tool} reads");
        }
    }

    #[test]
    fn opencode_strict_merges_existing_permission() {
        let cfg = serde_json::json!({
            "instructions": [],
            "permission": { "edit": "deny", "bash": "ask" }
        });
        let patched = opencode_with_hook(cfg, true);
        let permission = patched["permission"].as_object().unwrap();
        assert_eq!(permission["edit"], "deny", "existing permission rules must survive strict");
        assert_eq!(permission["read"], "ask");
        assert_eq!(permission["grep"], "ask");
        assert_eq!(permission["bash"], "ask");
    }

    #[test]
    fn claude_strict_settings_gate_first_read() {
        let settings = claude_strict_settings();
        let ask = settings["permissions"]["ask"].as_array().unwrap();
        assert!(ask.iter().any(|v| v.as_str() == Some("Read(//**)")));
        assert!(ask.iter().any(|v| v.as_str() == Some("Bash(*)")));
    }

    #[test]
    fn enforcement_capability_known_platforms() {
        assert!(platform_supports_enforcement("opencode"));
        assert!(platform_supports_enforcement("claude"));
        assert!(!platform_supports_enforcement("kiro"));
        assert!(!platform_supports_enforcement("codex"));
        assert!(!platform_supports_enforcement("cursor"));
        assert!(!platform_supports_enforcement("antigravity"));
    }
}
