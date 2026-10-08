use serde_yaml::Value;

use crate::models::RECORD_SCHEMA_VERSION;

const SCHEMA_VERSION_KEY: &str = "schema_version";
const STATE_KEY: &str = "state";

pub fn record_state_text(body: &str) -> Option<String> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return None;
    }
    let value: Value = serde_yaml::from_str(trimmed).ok()?;
    let version = value.get(SCHEMA_VERSION_KEY)?.as_u64()?;
    if version != u64::from(RECORD_SCHEMA_VERSION) {
        return None;
    }
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
    fn tolerates_extra_keys_before_schema_version() {
        let body = "\n\nnoise: value\nid: wiki:x\nschema_version: 1\nstate: |-\n  prose\nquestions: []\n";
        let state = record_state_text(body).expect("state must be found");
        assert_eq!(state, "prose");
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
