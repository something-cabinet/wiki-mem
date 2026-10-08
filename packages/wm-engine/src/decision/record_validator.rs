use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::models::PageType;

use super::canonical_questions::{canonical_questions, is_record_bearing};
use super::decision_error_model::DecisionError;
use crate::models::{AnswerValue, DecisionRecord, QType, Question, RECORD_SCHEMA_VERSION};

pub const MAX_QUESTIONS: usize = 32;
pub const MIN_OPTIONS: usize = 2;
pub const MAX_OPTIONS: usize = 20;
pub const MIN_LEVELS: usize = 2;
pub const MAX_LEVELS: usize = 10;
pub const MIN_INSTRUCTIONS_CHARS: usize = 1;
pub const MAX_INSTRUCTIONS_CHARS: usize = 240;
pub const MAX_QUESTION_ID_CHARS: usize = 48;

pub fn validate_record(page_type: &PageType, record: &DecisionRecord) -> Result<(), DecisionError> {
    if !is_record_bearing(page_type) {
        return Err(DecisionError::NotRecordBearing(
            page_type.as_str().to_owned(),
        ));
    }
    if record.schema_version != RECORD_SCHEMA_VERSION {
        return Err(DecisionError::UnsupportedSchemaVersion(
            record.schema_version,
        ));
    }
    if record.state.trim().is_empty() {
        return Err(DecisionError::EmptyState);
    }
    validate_question_count(record.questions.len())?;
    validate_questions(&record.questions)?;
    validate_answers(&record.questions, &record.answers)?;
    validate_canonical(page_type, record)
}

fn validate_question_count(count: usize) -> Result<(), DecisionError> {
    match count {
        0 => Err(DecisionError::InvalidQuestionCount { count }),
        count if count > MAX_QUESTIONS => Err(DecisionError::InvalidQuestionCount { count }),
        _ => Ok(()),
    }
}

fn validate_questions(questions: &[Question]) -> Result<(), DecisionError> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for question in questions {
        validate_question_id(question)?;
        if !seen.insert(question.id.as_str()) {
            return Err(DecisionError::DuplicateQuestionId(question.id.clone()));
        }
        validate_instructions(question)?;
        validate_multi(question)?;
        validate_question_vocabulary(question)?;
    }
    Ok(())
}

fn validate_question_id(question: &Question) -> Result<(), DecisionError> {
    if is_valid_question_id(&question.id) {
        return Ok(());
    }
    Err(DecisionError::InvalidQuestionId(question.id.clone()))
}

fn validate_instructions(question: &Question) -> Result<(), DecisionError> {
    let instructions = &question.instructions;
    let single_line = !instructions.contains('\n') && !instructions.contains('\r');
    let length = instructions.chars().count();
    let length_ok = (MIN_INSTRUCTIONS_CHARS..=MAX_INSTRUCTIONS_CHARS).contains(&length);
    if single_line && length_ok {
        return Ok(());
    }
    Err(DecisionError::InvalidInstructions {
        id: question.id.clone(),
    })
}

fn validate_multi(question: &Question) -> Result<(), DecisionError> {
    if !question.multi || question.qtype == QType::Choice {
        return Ok(());
    }
    Err(DecisionError::MultiOnNonChoice(question.id.clone()))
}

fn validate_question_vocabulary(question: &Question) -> Result<(), DecisionError> {
    match question.qtype {
        QType::Choice => validate_options(question),
        QType::Score => validate_levels(question),
        QType::Noul => Ok(()),
    }
}

fn validate_options(question: &Question) -> Result<(), DecisionError> {
    let options = &question.options;
    if !(MIN_OPTIONS..=MAX_OPTIONS).contains(&options.len()) {
        return Err(DecisionError::InvalidOptionCount {
            id: question.id.clone(),
        });
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for option in options {
        if !seen.insert(option.as_str()) {
            return Err(DecisionError::DuplicateOption {
                id: question.id.clone(),
                option: option.clone(),
            });
        }
    }
    Ok(())
}

fn validate_levels(question: &Question) -> Result<(), DecisionError> {
    let levels = &question.levels;
    if !(MIN_LEVELS..=MAX_LEVELS).contains(&levels.len()) {
        return Err(DecisionError::InvalidLevelCount {
            id: question.id.clone(),
        });
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for level in levels {
        if !seen.insert(level.as_str()) {
            return Err(DecisionError::DuplicateLevel {
                id: question.id.clone(),
                level: level.clone(),
            });
        }
    }
    Ok(())
}

fn validate_answers(
    questions: &[Question],
    answers: &BTreeMap<String, AnswerValue>,
) -> Result<(), DecisionError> {
    for (id, value) in answers {
        let Some(question) = questions.iter().find(|question| question.id == *id) else {
            return Err(DecisionError::UnknownAnswerKey(id.clone()));
        };
        if answer_matches(question, value) {
            continue;
        }
        return Err(DecisionError::InvalidAnswerValue { id: id.clone() });
    }
    Ok(())
}

fn answer_matches(question: &Question, value: &AnswerValue) -> bool {
    match (question.qtype, question.multi, value) {
        (QType::Choice, true, AnswerValue::Labels(labels)) => {
            labels.iter().all(|label| question.options.contains(label))
        }
        (QType::Choice, false, AnswerValue::Label(label)) => question.options.contains(label),
        (QType::Score, false, AnswerValue::Label(label)) => question.levels.contains(label),
        (QType::Noul, false, AnswerValue::Bool(_)) => true,
        _ => false,
    }
}

fn validate_canonical(page_type: &PageType, record: &DecisionRecord) -> Result<(), DecisionError> {
    let canonical = canonical_questions(page_type);
    let mismatch = canonical.len() != record.questions.len()
        || canonical
            .iter()
            .zip(record.questions.iter())
            .any(|(expected, actual)| !expected.same_shape(actual));
    if mismatch {
        return Err(DecisionError::CanonicalMismatch(
            page_type.as_str().to_owned(),
        ));
    }
    Ok(())
}

pub fn is_valid_question_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    let mut length = 1usize;
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return false;
        }
        length = length.saturating_add(1);
    }
    length <= MAX_QUESTION_ID_CHARS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::canonical_questions::canonical_questions;
    use crate::models::Question;

    fn noul_question(id: &str) -> Question {
        Question {
            id: id.to_owned(),
            qtype: QType::Noul,
            instructions: "A positive declarative statement.".to_owned(),
            multi: false,
            options: Vec::new(),
            levels: Vec::new(),
        }
    }

    fn valid_record(page_type: &PageType) -> DecisionRecord {
        DecisionRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            state: "normalized prose".to_owned(),
            questions: canonical_questions(page_type),
            answers: BTreeMap::new(),
        }
    }

    #[test]
    fn accepts_a_canonical_record_with_no_answers() {
        for page_type in [
            PageType::Decision,
            PageType::Rule,
            PageType::Pattern,
            PageType::Concept,
            PageType::Howto,
            PageType::Reference,
            PageType::Memory,
            PageType::Task,
            PageType::Spec,
            PageType::Core,
            PageType::Note,
        ] {
            let record = valid_record(&page_type);
            assert!(
                validate_record(&page_type, &record).is_ok(),
                "canonical {page_type:?} record should validate"
            );
        }
    }

    #[test]
    fn accepts_valid_answers_by_type() {
        let mut record = valid_record(&PageType::Decision);
        record.answers.insert(
            "outcome".to_owned(),
            AnswerValue::Label("adopted".to_owned()),
        );
        record.answers.insert(
            "confidence".to_owned(),
            AnswerValue::Label("high".to_owned()),
        );
        record
            .answers
            .insert("reversibility".to_owned(), AnswerValue::Bool(true));
        assert!(validate_record(&PageType::Decision, &record).is_ok());

        let mut multi = valid_record(&PageType::Pattern);
        multi.answers.insert(
            "problem_kind".to_owned(),
            AnswerValue::Label("architecture".to_owned()),
        );
        assert!(validate_record(&PageType::Pattern, &multi).is_ok());
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut record = valid_record(&PageType::Concept);
        record.schema_version = 2;
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::UnsupportedSchemaVersion(2))
        );
    }

    #[test]
    fn rejects_empty_state() {
        let mut record = valid_record(&PageType::Concept);
        record.state = "   ".to_owned();
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::EmptyState)
        );
    }

    #[test]
    fn rejects_questions_that_do_not_match_the_type() {
        let record = valid_record(&PageType::Concept);
        assert_eq!(
            validate_record(&PageType::Rule, &record),
            Err(DecisionError::CanonicalMismatch("rule".to_owned()))
        );
    }

    #[test]
    fn rejects_zero_and_too_many_questions() {
        let mut empty = valid_record(&PageType::Concept);
        empty.questions.clear();
        assert_eq!(
            validate_record(&PageType::Concept, &empty),
            Err(DecisionError::InvalidQuestionCount { count: 0 })
        );

        let mut many = valid_record(&PageType::Concept);
        many.questions = (0usize..33)
            .map(|index| noul_question(&format!("q{index}")))
            .collect();
        assert_eq!(
            validate_record(&PageType::Concept, &many),
            Err(DecisionError::InvalidQuestionCount { count: 33 })
        );
    }

    #[test]
    fn rejects_invalid_question_id() {
        let mut record = valid_record(&PageType::Concept);
        record.questions[0].id = "Bad".to_owned();
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::InvalidQuestionId("Bad".to_owned()))
        );

        let too_long = format!("a{}", "b".repeat(48));
        assert!(!is_valid_question_id(&too_long));
        assert!(is_valid_question_id("a"));
        assert!(is_valid_question_id("outcome_1"));
    }

    #[test]
    fn rejects_invalid_instructions() {
        let mut empty = valid_record(&PageType::Concept);
        empty.questions[0].instructions = String::new();
        assert_eq!(
            validate_record(&PageType::Concept, &empty),
            Err(DecisionError::InvalidInstructions {
                id: empty.questions[0].id.clone()
            })
        );

        let mut multiline = valid_record(&PageType::Concept);
        multiline.questions[0].instructions = "line one\nline two".to_owned();
        assert!(matches!(
            validate_record(&PageType::Concept, &multiline),
            Err(DecisionError::InvalidInstructions { .. })
        ));

        let mut long = valid_record(&PageType::Concept);
        long.questions[0].instructions = "x".repeat(241);
        assert!(matches!(
            validate_record(&PageType::Concept, &long),
            Err(DecisionError::InvalidInstructions { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_option_count() {
        let mut record = valid_record(&PageType::Concept);
        record.questions[0].options = vec!["only".to_owned()];
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::InvalidOptionCount {
                id: record.questions[0].id.clone()
            })
        );
    }

    #[test]
    fn rejects_duplicate_options() {
        let mut record = valid_record(&PageType::Concept);
        record.questions[0].options = vec!["a".to_owned(), "a".to_owned()];
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::DuplicateOption {
                id: record.questions[0].id.clone(),
                option: "a".to_owned()
            })
        );
    }

    #[test]
    fn rejects_out_of_range_level_count_and_duplicates() {
        let mut too_few = valid_record(&PageType::Concept);
        too_few.questions[2].levels = vec!["only".to_owned()];
        assert_eq!(
            validate_record(&PageType::Concept, &too_few),
            Err(DecisionError::InvalidLevelCount {
                id: too_few.questions[2].id.clone()
            })
        );

        let mut duplicate = valid_record(&PageType::Concept);
        duplicate.questions[2].levels = vec!["a".to_owned(), "a".to_owned()];
        assert_eq!(
            validate_record(&PageType::Concept, &duplicate),
            Err(DecisionError::DuplicateLevel {
                id: duplicate.questions[2].id.clone(),
                level: "a".to_owned()
            })
        );
    }

    #[test]
    fn rejects_multi_on_non_choice() {
        let mut record = valid_record(&PageType::Concept);
        record.questions[2].multi = true;
        assert_eq!(
            validate_record(&PageType::Concept, &record),
            Err(DecisionError::MultiOnNonChoice(
                record.questions[2].id.clone()
            ))
        );
    }

    #[test]
    fn rejects_unknown_answer_key() {
        let mut record = valid_record(&PageType::Decision);
        record
            .answers
            .insert("mystery".to_owned(), AnswerValue::Bool(true));
        assert_eq!(
            validate_record(&PageType::Decision, &record),
            Err(DecisionError::UnknownAnswerKey("mystery".to_owned()))
        );
    }

    #[test]
    fn rejects_invalid_answer_values() {
        let mut wrong_label = valid_record(&PageType::Decision);
        wrong_label.answers.insert(
            "outcome".to_owned(),
            AnswerValue::Label("nonsense".to_owned()),
        );
        assert_eq!(
            validate_record(&PageType::Decision, &wrong_label),
            Err(DecisionError::InvalidAnswerValue {
                id: "outcome".to_owned()
            })
        );

        let mut bool_for_noul = valid_record(&PageType::Decision);
        bool_for_noul
            .answers
            .insert("confidence".to_owned(), AnswerValue::Bool(true));
        assert_eq!(
            validate_record(&PageType::Decision, &bool_for_noul),
            Err(DecisionError::InvalidAnswerValue {
                id: "confidence".to_owned()
            })
        );

        let mut list_for_single = valid_record(&PageType::Decision);
        list_for_single.answers.insert(
            "outcome".to_owned(),
            AnswerValue::Labels(vec!["adopted".to_owned()]),
        );
        assert_eq!(
            validate_record(&PageType::Decision, &list_for_single),
            Err(DecisionError::InvalidAnswerValue {
                id: "outcome".to_owned()
            })
        );

        let mut label_for_multi = valid_record(&PageType::Pattern);
        label_for_multi.answers.insert(
            "problem_kind".to_owned(),
            AnswerValue::Label("architecture".to_owned()),
        );
        assert!(validate_record(&PageType::Pattern, &label_for_multi).is_ok());
    }

    #[test]
    fn rejects_multi_answer_list_for_single_choice() {
        let mut record = valid_record(&PageType::Decision);
        record.answers.insert(
            "outcome".to_owned(),
            AnswerValue::Labels(vec!["adopted".to_owned(), "rejected".to_owned()]),
        );
        assert_eq!(
            validate_record(&PageType::Decision, &record),
            Err(DecisionError::InvalidAnswerValue {
                id: "outcome".to_owned()
            })
        );
    }

    #[test]
    fn rejects_canonical_mismatch() {
        let mut record = valid_record(&PageType::Decision);
        record.questions[0].options = vec!["adopted".to_owned(), "nope".to_owned()];
        assert_eq!(
            validate_record(&PageType::Decision, &record),
            Err(DecisionError::CanonicalMismatch("decision".to_owned()))
        );
    }

    #[test]
    fn rejects_canonical_reordering() {
        let mut record = valid_record(&PageType::Decision);
        record.questions.swap(0, 1);
        assert_eq!(
            validate_record(&PageType::Decision, &record),
            Err(DecisionError::CanonicalMismatch("decision".to_owned()))
        );
    }

    #[test]
    fn ignores_instructions_in_canonical_equality() {
        let mut record = valid_record(&PageType::Decision);
        record.questions[0].instructions = "A custom but valid prompt.".to_owned();
        assert!(validate_record(&PageType::Decision, &record).is_ok());
    }
}
