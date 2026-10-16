use wm_engine::{is_record_bearing, parse_record, PageType};

const MIGRATION_COMMAND: &str = "wm page migrate-records --only <path>";
const SCHEMA_REFERENCE: &str = "@wiki/reference/typed-decision-record-schema";

pub fn record_format_hint(page_type: &PageType, body: &str) -> Option<String> {
    if !is_record_bearing(page_type) {
        return None;
    }
    if parse_record(page_type, body).is_ok() {
        return None;
    }
    Some(format!(
        "This is a record-bearing page type but its body is not a typed-decision record. Convert it with `{MIGRATION_COMMAND}`, or author it as a record per {SCHEMA_REFERENCE}."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DECISION: &str = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected, deferred, superseded, abandoned]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.\n  - id: confidence\n    type: score\n    instructions: How strong is the recorded justification for the selected outcome?\n    levels: [low, medium, high]\n  - id: impact\n    type: choice\n    instructions: How wide is the blast radius of this decision?\n    options: [local, component, system, project-wide]\nanswers: {}\n";

    #[test]
    fn hints_on_prose_body_for_record_bearing_type() {
        let hint = record_format_hint(&PageType::Concept, "## Context\n\nprose\n")
            .expect("prose record-bearing page must hint");
        assert!(hint.contains("migrate-records"));
        assert!(hint.contains("typed-decision-record-schema"));
    }

    #[test]
    fn hints_on_empty_body_for_record_bearing_type() {
        assert!(record_format_hint(&PageType::Decision, "").is_some());
    }

    #[test]
    fn hints_on_malformed_record() {
        let body = "schema_version: 1\nquestions: []\n";
        assert!(record_format_hint(&PageType::Concept, body).is_some());
    }

    #[test]
    fn no_hint_for_valid_record() {
        assert!(record_format_hint(&PageType::Decision, VALID_DECISION).is_none());
    }

    #[test]
    fn hints_on_prose_for_previously_excluded_types() {
        for page_type in [
            PageType::Rule,
            PageType::Core,
            PageType::Memory,
            PageType::Task,
            PageType::Spec,
            PageType::Note,
        ] {
            assert!(
                record_format_hint(&page_type, "## Prose\n\nbody\n").is_some(),
                "{page_type:?} must hint"
            );
        }
    }
}
