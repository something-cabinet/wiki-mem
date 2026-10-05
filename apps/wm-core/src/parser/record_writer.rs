use wm_engine::{AnswerValue, DecisionRecord};

use crate::page::helpers::yaml_helper::{set_yaml_block, yaml_scalar};

pub fn write_record_answers(content: &str, record: &DecisionRecord) -> String {
    set_yaml_block(content, "answers", &render_answers_block(record))
}

pub fn render_answers_block(record: &DecisionRecord) -> String {
    if record.answers.is_empty() {
        return "answers: {}\n".to_owned();
    }
    let mut out = String::from("answers:\n");
    for (key, value) in &record.answers {
        out.push_str(&format!("  {key}: {}\n", render_answer_value(value)));
    }
    out
}

fn render_answer_value(value: &AnswerValue) -> String {
    match value {
        AnswerValue::Bool(flag) => flag.to_string(),
        AnswerValue::Label(label) => yaml_scalar(label),
        AnswerValue::Labels(labels) => {
            let rendered = labels
                .iter()
                .map(|label| yaml_scalar(label))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{rendered}]")
        }
    }
}

pub type Answers = std::collections::BTreeMap<String, AnswerValue>;

#[cfg(test)]
mod tests {
    use super::*;
    use wm_engine::{
        canonical_questions, parse_record, DecisionRecord, PageType, RECORD_SCHEMA_VERSION,
    };

    const FRONTMATTER: &str =
        "---\ntitle: Choose a database\nid: \"wiki:decisions:choose-db\"\ntype: decision\nstatus: approved\n---\n";

    fn answers(pairs: &[(&str, AnswerValue)]) -> Answers {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect()
    }

    fn representative(page_type: &PageType) -> DecisionRecord {
        let answers = match page_type {
            PageType::Decision => answers(&[
                ("outcome", AnswerValue::Label("adopted".to_owned())),
                ("reversibility", AnswerValue::Bool(false)),
                ("confidence", AnswerValue::Label("high".to_owned())),
                ("impact", AnswerValue::Label("system".to_owned())),
            ]),
            PageType::Pattern => answers(&[
                (
                    "problem_kind",
                    AnswerValue::Label("architecture".to_owned()),
                ),
                ("preconditions_required", AnswerValue::Bool(true)),
                ("complexity", AnswerValue::Label("moderate".to_owned())),
                ("language_specific", AnswerValue::Bool(false)),
            ]),
            PageType::Concept => answers(&[
                ("kind", AnswerValue::Label("concept".to_owned())),
                ("category", AnswerValue::Label("storage".to_owned())),
                ("maturity", AnswerValue::Label("stable".to_owned())),
                ("code_referenced", AnswerValue::Bool(true)),
            ]),
            PageType::Howto => answers(&[
                ("task_kind", AnswerValue::Label("setup".to_owned())),
                ("prerequisites_complete", AnswerValue::Bool(true)),
                ("difficulty", AnswerValue::Label("beginner".to_owned())),
                ("has_verification", AnswerValue::Bool(false)),
            ]),
            PageType::Reference => answers(&[
                ("kind", AnswerValue::Label("api".to_owned())),
                (
                    "surface",
                    AnswerValue::Labels(vec!["mcp-tool".to_owned(), "cli-command".to_owned()]),
                ),
                ("stability", AnswerValue::Label("stable".to_owned())),
                ("has_examples", AnswerValue::Bool(true)),
            ]),
            _ => Answers::new(),
        };
        DecisionRecord {
            schema_version: RECORD_SCHEMA_VERSION,
            state: "## Context\n\nChose SQLite over Postgres for the local store.".to_owned(),
            questions: canonical_questions(page_type),
            answers,
        }
    }

    fn fixture(page_type: &PageType) -> (String, DecisionRecord) {
        let record = representative(page_type);
        let body = serde_yaml::to_string(&record).expect("record should serialize");
        (format!("{FRONTMATTER}\n{body}"), record)
    }

    #[test]
    fn round_trip_identity_for_each_record_bearing_type() {
        for page_type in [
            PageType::Decision,
            PageType::Pattern,
            PageType::Concept,
            PageType::Howto,
            PageType::Reference,
        ] {
            let (content, expected) = fixture(&page_type);
            let before = content
                .split("\nanswers:")
                .next()
                .expect("fixture has answers")
                .to_owned();

            let written = write_record_answers(&content, &expected);

            assert!(
                written.contains("id: \"wiki:decisions:choose-db\""),
                "frontmatter id must stay double-quoted for {page_type:?}"
            );
            let after = written
                .split("\nanswers:")
                .next()
                .expect("written has answers")
                .to_owned();
            assert_eq!(
                before, after,
                "state and questions must be immutable for {page_type:?}"
            );

            let body = written
                .split_once("---\n")
                .and_then(|(_, rest)| rest.split_once("---\n"))
                .map(|(_, body)| body.trim().to_owned())
                .expect("written page must retain frontmatter");
            let reparsed = parse_record(&page_type, &body).expect("written body must parse");
            assert_eq!(
                reparsed, expected,
                "record must round-trip for {page_type:?}"
            );
        }
    }

    #[test]
    fn rewrite_replaces_the_answers_block_in_place() {
        let (content, mut record) = fixture(&PageType::Decision);
        record.answers.insert(
            "outcome".to_owned(),
            AnswerValue::Label("rejected".to_owned()),
        );
        record
            .answers
            .insert("impact".to_owned(), AnswerValue::Label("local".to_owned()));

        let written = write_record_answers(&content, &record);

        assert_eq!(
            written.matches("answers:").count(),
            1,
            "answers block must be replaced, not appended"
        );
        let body = written
            .split_once("---\n")
            .and_then(|(_, rest)| rest.split_once("---\n"))
            .map(|(_, body)| body.trim().to_owned())
            .expect("written page must retain frontmatter");
        let reparsed = parse_record(&PageType::Decision, &body).expect("body must parse");
        assert_eq!(
            reparsed.answers.get("outcome"),
            Some(&AnswerValue::Label("rejected".to_owned()))
        );
    }

    #[test]
    fn renders_empty_answers_as_empty_mapping() {
        let mut record = representative(&PageType::Concept);
        record.answers.clear();
        assert_eq!(render_answers_block(&record), "answers: {}\n");
    }

    #[test]
    fn writes_answers_when_the_block_is_absent() {
        let mut empty = representative(&PageType::Decision);
        empty.answers.clear();
        let body = serde_yaml::to_string(&empty).expect("record should serialize");
        let without_answers = body
            .lines()
            .filter(|line| !line.starts_with("answers:"))
            .collect::<Vec<_>>()
            .join("\n");
        let content = format!("{FRONTMATTER}\n{without_answers}\n");
        let record = representative(&PageType::Decision);

        let written = write_record_answers(&content, &record);
        let body = written
            .split_once("---\n")
            .and_then(|(_, rest)| rest.split_once("---\n"))
            .map(|(_, body)| body.trim().to_owned())
            .expect("written page must retain frontmatter");
        let reparsed = parse_record(&PageType::Decision, &body).expect("body must parse");
        assert_eq!(reparsed, record);
    }
}
