use serde_yaml::Value;

use crate::models::RECORD_SCHEMA_VERSION;

const SCHEMA_VERSION_KEY: &str = "schema_version";
const STATE_KEY: &str = "state";

/// Extract the `state` block scalar from a typed-decision record body.
///
/// Returns `None` for prose bodies so callers fall back to the raw body.
/// Detection is deliberately lenient: it keys off the FR-9 idempotency marker
/// (first non-blank line `schema_version: <current>`) plus a string `state`
/// field. Full record validation lives in [`super::parse_record`].
pub fn record_state_text(body: &str) -> Option<String> {
    let first_line = body.lines().find(|line| !line.trim().is_empty())?;
    let (key, value) = first_line.trim().split_once(':')?;
    if key.trim() != SCHEMA_VERSION_KEY {
        return None;
    }
    if value.trim().parse::<u32>().ok()? != RECORD_SCHEMA_VERSION {
        return None;
    }
    let value: Value = serde_yaml::from_str(body.trim()).ok()?;
    value.get(STATE_KEY)?.as_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECORD_BODY: &str = "schema_version: 1\nstate: |-\n  ## Context\n\n  Chose SQLite over Postgres.\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected]\nanswers:\n  outcome: adopted\n";

    #[test]
    fn extracts_deindented_state_from_record_body() {
        let state = record_state_text(RECORD_BODY).expect("record body must yield state");
        assert!(state.starts_with("## Context"));
        assert!(state.contains("Chose SQLite over Postgres."));
        assert!(!state.contains("schema_version"));
    }

    #[test]
    fn ignores_leading_blank_lines() {
        let body = format!("\n\n{RECORD_BODY}");
        assert!(record_state_text(&body).is_some());
    }

    #[test]
    fn returns_none_for_prose_body() {
        assert!(record_state_text("## Context\n\nJust prose.\n").is_none());
    }

    #[test]
    fn returns_none_for_unsupported_schema_version() {
        let body = "schema_version: 2\nstate: |-\n  prose\nquestions: []\n";
        assert!(record_state_text(body).is_none());
    }

    #[test]
    fn returns_none_when_state_is_missing() {
        let body = "schema_version: 1\nquestions: []\n";
        assert!(record_state_text(body).is_none());
    }

    #[test]
    fn returns_none_when_state_is_not_a_string() {
        let body = "schema_version: 1\nstate: 42\nquestions: []\n";
        assert!(record_state_text(body).is_none());
    }
}
