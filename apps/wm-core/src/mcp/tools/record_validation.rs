use wm_engine::{is_record_bearing, parse_record, validate_record, DecisionError, PageType};

const RECORD_FIELD_PREFIX: &str = "record";
const STATE_FIELD: &str = "record.state";
const QUESTIONS_FIELD: &str = "record.questions";
const SCHEMA_VERSION_FIELD: &str = "record.schema_version";
const QUESTIONS_INDEX_PREFIX: &str = "record.questions[";
const ANSWERS_INDEX_PREFIX: &str = "record.answers.";
const VALID_QUESTION_TYPES: [&str; 3] = ["choice", "score", "noul"];

/// A record-envelope validation error attributed to a page field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordValidationError {
    /// Field path within the record (e.g. `record.questions[outcome].id`).
    pub field: String,
    /// Human-readable error message.
    pub message: String,
}

/// Validates a record-bearing page body, returning every envelope error.
///
/// Non-record-bearing types and prose bodies yield no errors: the record
/// envelope is only enforced once a body opts in via `schema_version`.
pub fn validate_page_record(page_type: &PageType, body: &str) -> Vec<RecordValidationError> {
    if !is_record_bearing(page_type) {
        return Vec::new();
    }
    if !looks_like_record(body) {
        return Vec::new();
    }
    let record = match parse_record(page_type, body) {
        Ok(record) => record,
        Err(error) => return vec![map_parse_error(&error, body)],
    };
    match validate_record(page_type, &record) {
        Ok(()) => Vec::new(),
        Err(error) => vec![map_validate_error(&error)],
    }
}

fn looks_like_record(body: &str) -> bool {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.starts_with("schema_version:"))
        .unwrap_or(false)
}

fn map_parse_error(error: &DecisionError, body: &str) -> RecordValidationError {
    if let Some(id) = bad_question_type_id(error, body) {
        return RecordValidationError {
            field: format!("{QUESTIONS_INDEX_PREFIX}{id}].type"),
            message: error.to_string(),
        };
    }
    match error {
        DecisionError::MissingField(key) => RecordValidationError {
            field: format!("{RECORD_FIELD_PREFIX}.{key}"),
            message: error.to_string(),
        },
        DecisionError::UnknownTopLevelKey(key) => RecordValidationError {
            field: format!("{RECORD_FIELD_PREFIX}.{key}"),
            message: error.to_string(),
        },
        DecisionError::UnsupportedSchemaVersion(_) => RecordValidationError {
            field: SCHEMA_VERSION_FIELD.to_owned(),
            message: error.to_string(),
        },
        DecisionError::EmptyState => RecordValidationError {
            field: STATE_FIELD.to_owned(),
            message: error.to_string(),
        },
        DecisionError::InvalidYaml(_)
        | DecisionError::NotAMapping
        | DecisionError::AnchorOrAlias
        | DecisionError::DocumentSeparator => RecordValidationError {
            field: RECORD_FIELD_PREFIX.to_owned(),
            message: error.to_string(),
        },
        _ => RecordValidationError {
            field: RECORD_FIELD_PREFIX.to_owned(),
            message: error.to_string(),
        },
    }
}

fn bad_question_type_id(error: &DecisionError, body: &str) -> Option<String> {
    let DecisionError::InvalidYaml(message) = error else {
        return None;
    };
    if !message.contains("unknown variant") {
        return None;
    }
    let value: serde_yaml::Value = serde_yaml::from_str(body.trim()).ok()?;
    let questions = value.get("questions")?.as_sequence()?;
    for question in questions {
        let id = question.get("id")?.as_str()?;
        let qtype = question.get("type")?.as_str()?;
        if !VALID_QUESTION_TYPES.contains(&qtype) {
            return Some(id.to_owned());
        }
    }
    None
}

fn map_validate_error(error: &DecisionError) -> RecordValidationError {
    match error {
        DecisionError::UnsupportedSchemaVersion(_) => RecordValidationError {
            field: SCHEMA_VERSION_FIELD.to_owned(),
            message: error.to_string(),
        },
        DecisionError::EmptyState => RecordValidationError {
            field: STATE_FIELD.to_owned(),
            message: error.to_string(),
        },
        DecisionError::InvalidQuestionCount { .. } => RecordValidationError {
            field: QUESTIONS_FIELD.to_owned(),
            message: error.to_string(),
        },
        DecisionError::DuplicateQuestionId(id) => RecordValidationError {
            field: format!("{QUESTIONS_INDEX_PREFIX}{id}].id"),
            message: error.to_string(),
        },
        DecisionError::InvalidQuestionId(id) => RecordValidationError {
            field: format!("{QUESTIONS_INDEX_PREFIX}{id}].id"),
            message: error.to_string(),
        },
        DecisionError::InvalidInstructions { id } => RecordValidationError {
            field: format!("{QUESTIONS_INDEX_PREFIX}{id}].instructions"),
            message: error.to_string(),
        },
        DecisionError::InvalidOptionCount { id } | DecisionError::DuplicateOption { id, .. } => {
            RecordValidationError {
                field: format!("{QUESTIONS_INDEX_PREFIX}{id}].options"),
                message: error.to_string(),
            }
        }
        DecisionError::InvalidLevelCount { id } | DecisionError::DuplicateLevel { id, .. } => {
            RecordValidationError {
                field: format!("{QUESTIONS_INDEX_PREFIX}{id}].levels"),
                message: error.to_string(),
            }
        }
        DecisionError::MultiOnNonChoice(id) => RecordValidationError {
            field: format!("{QUESTIONS_INDEX_PREFIX}{id}].multi"),
            message: error.to_string(),
        },
        DecisionError::UnknownAnswerKey(id) => RecordValidationError {
            field: format!("{ANSWERS_INDEX_PREFIX}{id}"),
            message: error.to_string(),
        },
        DecisionError::InvalidAnswerValue { id } => RecordValidationError {
            field: format!("{ANSWERS_INDEX_PREFIX}{id}"),
            message: error.to_string(),
        },
        DecisionError::CanonicalMismatch(_) => RecordValidationError {
            field: QUESTIONS_FIELD.to_owned(),
            message: error.to_string(),
        },
        _ => RecordValidationError {
            field: RECORD_FIELD_PREFIX.to_owned(),
            message: error.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DECISION: &str = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected, deferred, superseded, abandoned]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply without data migration or cross-module breakage.\n  - id: confidence\n    type: score\n    instructions: How strong is the recorded justification for the selected outcome?\n    levels: [low, medium, high]\n  - id: impact\n    type: choice\n    instructions: How wide is the blast radius of this decision?\n    options: [local, component, system, project-wide]\nanswers: {}\n";

    fn fields(page_type: &PageType, body: &str) -> Vec<String> {
        validate_page_record(page_type, body)
            .into_iter()
            .map(|error| error.field)
            .collect()
    }

    #[test]
    fn accepts_a_valid_record() {
        assert!(validate_page_record(&PageType::Decision, VALID_DECISION).is_empty());
    }

    #[test]
    fn ignores_prose_bodies() {
        assert!(validate_page_record(&PageType::Concept, "## Context\n\nprose\n").is_empty());
    }

    #[test]
    fn ignores_non_record_bearing_types() {
        assert!(validate_page_record(&PageType::Rule, VALID_DECISION).is_empty());
    }

    #[test]
    fn reports_missing_state() {
        let body = "schema_version: 1\nquestions: []\n";
        assert_eq!(fields(&PageType::Concept, body), vec!["record.state"]);
    }

    #[test]
    fn reports_bad_question_id() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: Bad\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers: {}\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.questions[Bad].id"]
        );
    }

    #[test]
    fn reports_bad_instructions() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: \"\"\n    options: [a, b]\nanswers: {}\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.questions[outcome].instructions"]
        );
    }

    #[test]
    fn reports_option_count() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [only]\nanswers: {}\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.questions[outcome].options"]
        );
    }

    #[test]
    fn reports_level_count() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: confidence\n    type: score\n    instructions: How strong?\n    levels: [only]\nanswers: {}\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.questions[confidence].levels"]
        );
    }

    #[test]
    fn reports_multi_on_non_choice() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: confidence\n    type: score\n    multi: true\n    instructions: How strong?\n    levels: [low, high]\nanswers: {}\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.questions[confidence].multi"]
        );
    }

    #[test]
    fn reports_unknown_answer_key() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers:\n  mystery: a\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.answers.mystery"]
        );
    }

    #[test]
    fn reports_invalid_answer_value() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers:\n  outcome: nonsense\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.answers.outcome"]
        );
    }

    #[test]
    fn reports_canonical_mismatch() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions:\n  - id: outcome\n    type: choice\n    instructions: Pick one.\n    options: [a, b]\nanswers: {}\n";
        assert_eq!(fields(&PageType::Decision, body), vec!["record.questions"]);
    }

    #[test]
    fn reports_unsupported_schema_version() {
        let body = "schema_version: 2\nstate: |-\n  prose\nquestions: []\n";
        assert_eq!(
            fields(&PageType::Concept, body),
            vec!["record.schema_version"]
        );
    }
}
