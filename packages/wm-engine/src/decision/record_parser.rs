use serde_yaml::Value;

use crate::models::PageType;

use super::canonical_questions::is_record_bearing;
use super::decision_error_model::DecisionError;
use crate::models::{DecisionRecord, RECORD_SCHEMA_VERSION};

const ALLOWED_TOP_LEVEL_KEYS: [&str; 4] = ["schema_version", "state", "questions", "answers"];
const REQUIRED_TOP_LEVEL_KEYS: [&str; 3] = ["schema_version", "state", "questions"];

pub fn parse_record(page_type: &PageType, body: &str) -> Result<DecisionRecord, DecisionError> {
    if !is_record_bearing(page_type) {
        return Err(DecisionError::NotRecordBearing(
            page_type.as_str().to_owned(),
        ));
    }

    let body = body.trim();
    reject_anchors_and_separators(body)?;

    let value: Value = serde_yaml::from_str(body)
        .map_err(|error| DecisionError::InvalidYaml(error.to_string()))?;

    let Value::Mapping(mapping) = &value else {
        return Err(DecisionError::NotAMapping);
    };

    for key in mapping.keys() {
        let Some(key) = key.as_str() else {
            return Err(DecisionError::UnknownTopLevelKey("<non-string>".to_owned()));
        };
        if !ALLOWED_TOP_LEVEL_KEYS.contains(&key) {
            return Err(DecisionError::UnknownTopLevelKey(key.to_owned()));
        }
    }

    for key in REQUIRED_TOP_LEVEL_KEYS {
        if !mapping.contains_key(Value::String(key.to_owned())) {
            return Err(DecisionError::MissingField(key.to_owned()));
        }
    }

    let record: DecisionRecord = serde_yaml::from_value(value)
        .map_err(|error| DecisionError::InvalidYaml(error.to_string()))?;

    if record.schema_version != RECORD_SCHEMA_VERSION {
        return Err(DecisionError::UnsupportedSchemaVersion(
            record.schema_version,
        ));
    }
    if record.state.trim().is_empty() {
        return Err(DecisionError::EmptyState);
    }

    Ok(record)
}

fn reject_anchors_and_separators(body: &str) -> Result<(), DecisionError> {
    for raw_line in body.lines() {
        if raw_line.starts_with(' ') || raw_line.starts_with('\t') {
            continue;
        }
        let line = raw_line.trim_end();
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "..." {
            return Err(DecisionError::DocumentSeparator);
        }
        if trimmed.starts_with('&') || trimmed.starts_with('*') {
            return Err(DecisionError::AnchorOrAlias);
        }
        let Some((_key, rest)) = line.split_once(':') else {
            continue;
        };
        if rest
            .split_whitespace()
            .any(|token| token.starts_with('&') || token.starts_with('*'))
        {
            return Err(DecisionError::AnchorOrAlias);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnswerValue, QType};
    use std::collections::BTreeMap;

    fn body_with_questions(questions: &str, answers: &str) -> String {
        format!(
            "schema_version: 1\nstate: |-\n  prose here\nquestions:\n{questions}answers:\n{answers}"
        )
    }

    #[test]
    fn parses_choice_score_and_noul_questions() {
        let body = body_with_questions(
            "  - id: outcome\n    type: choice\n    instructions: What is the recorded outcome of this decision?\n    options: [adopted, rejected]\n  - id: confidence\n    type: score\n    instructions: How strong is the justification?\n    levels: [low, high]\n  - id: reversibility\n    type: noul\n    instructions: The decision can be reversed cheaply.\n",
            "  outcome: adopted\n  confidence: high\n  reversibility: true\n",
        );

        let record = parse_record(&PageType::Decision, &body).expect("record should parse");

        assert_eq!(record.schema_version, 1);
        assert_eq!(record.state, "prose here");
        assert_eq!(record.questions.len(), 3);
        assert_eq!(record.questions[0].qtype, QType::Choice);
        assert_eq!(record.questions[1].qtype, QType::Score);
        assert_eq!(record.questions[2].qtype, QType::Noul);
        assert_eq!(
            record.answers.get("outcome"),
            Some(&AnswerValue::Label("adopted".to_owned()))
        );
        assert_eq!(
            record.answers.get("reversibility"),
            Some(&AnswerValue::Bool(true))
        );
    }

    #[test]
    fn parses_multi_choice_answer_as_list() {
        let body = body_with_questions(
            "  - id: applies_to\n    type: choice\n    multi: true\n    instructions: Which surfaces?\n    options: [code, tests]\n",
            "  applies_to: [code, tests]\n",
        );

        let record = parse_record(&PageType::Pattern, &body).expect("record should parse");
        assert!(matches!(
            record.answers.get("applies_to"),
            Some(AnswerValue::Labels(labels)) if labels.len() == 2
        ));
    }

    #[test]
    fn accepts_empty_answers_map() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions: []\nanswers: {}\n";
        let record = parse_record(&PageType::Concept, body).expect("record should parse");
        assert!(record.answers.is_empty());
    }

    #[test]
    fn accepts_missing_answers_key() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions: []\n";
        let record = parse_record(&PageType::Concept, body).expect("record should parse");
        assert_eq!(record.answers, BTreeMap::new());
    }

    #[test]
    fn rejects_non_record_bearing_type() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions: []\n";
        let error = parse_record(&PageType::Rule, body).expect_err("rule must not parse");
        assert_eq!(error, DecisionError::NotRecordBearing("rule".to_owned()));
    }

    #[test]
    fn rejects_unknown_top_level_key() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions: []\nextra: nope\n";
        let error = parse_record(&PageType::Concept, body).expect_err("unknown key must fail");
        assert_eq!(error, DecisionError::UnknownTopLevelKey("extra".to_owned()));
    }

    #[test]
    fn rejects_missing_state() {
        let body = "schema_version: 1\nquestions: []\n";
        let error = parse_record(&PageType::Concept, body).expect_err("missing state must fail");
        assert_eq!(error, DecisionError::MissingField("state".to_owned()));
    }

    #[test]
    fn rejects_empty_state() {
        let body = "schema_version: 1\nstate: |-\nquestions: []\n";
        let error = parse_record(&PageType::Concept, body).expect_err("empty state must fail");
        assert_eq!(error, DecisionError::EmptyState);
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let body = "schema_version: 2\nstate: |-\n  prose\nquestions: []\n";
        let error = parse_record(&PageType::Concept, body).expect_err("version must fail");
        assert_eq!(error, DecisionError::UnsupportedSchemaVersion(2));
    }

    #[test]
    fn rejects_anchors_and_aliases() {
        let body = "schema_version: 1\nstate: |-\n  prose\nquestions: &anchor []\n";
        let error = parse_record(&PageType::Concept, body).expect_err("anchor must fail");
        assert_eq!(error, DecisionError::AnchorOrAlias);
    }

    #[test]
    fn rejects_document_separators() {
        let body = "schema_version: 1\nstate: |-\n  prose\n---\nquestions: []\n";
        let error = parse_record(&PageType::Concept, body).expect_err("separator must fail");
        assert_eq!(error, DecisionError::DocumentSeparator);
    }

    #[test]
    fn rejects_non_mapping_body() {
        let error =
            parse_record(&PageType::Concept, "- one\n- two\n").expect_err("sequence must fail");
        assert_eq!(error, DecisionError::NotAMapping);
    }
}
