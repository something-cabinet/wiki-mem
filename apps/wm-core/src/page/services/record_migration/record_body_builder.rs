use std::collections::BTreeMap;

use wm_engine::{canonical_questions, DecisionRecord, PageType, RECORD_SCHEMA_VERSION};

const SCHEMA_VERSION_LINE: &str = "schema_version: 1";
const STATE_KEY: &str = "state";
const QUESTIONS_KEY: &str = "questions";
const EMPTY_ANSWERS_LINE: &str = "answers: {}";
const STATE_BLOCK_INDICATOR: &str = "|-";
const STATE_BLOCK_INDENTED_INDICATOR: &str = "|2-";
const BLOCK_INDENT: usize = 2;

pub fn build_record_body(page_type: &PageType, state: &str) -> Result<String, String> {
    let record = DecisionRecord {
        schema_version: RECORD_SCHEMA_VERSION,
        state: state.to_owned(),
        questions: canonical_questions(page_type),
        answers: BTreeMap::new(),
    };
    let questions_yaml =
        serde_yaml::to_string(&record.questions).map_err(|error| error.to_string())?;

    let mut body = String::new();
    body.push_str(SCHEMA_VERSION_LINE);
    body.push('\n');
    body.push_str(&render_state_block(state)?);
    body.push_str(QUESTIONS_KEY);
    body.push_str(":\n");
    push_indented(&mut body, &questions_yaml);
    body.push_str(EMPTY_ANSWERS_LINE);
    body.push('\n');
    Ok(body)
}

fn render_state_block(state: &str) -> Result<String, String> {
    if state.contains('\r') {
        return Err("state contains a carriage return".to_owned());
    }
    let indicator = match first_content_line_is_indented(state) {
        true => STATE_BLOCK_INDENTED_INDICATOR,
        false => STATE_BLOCK_INDICATOR,
    };
    let mut block = format!("{STATE_KEY}: {indicator}\n");
    for line in state.split('\n') {
        if line.is_empty() {
            block.push('\n');
            continue;
        }
        block.push_str(&" ".repeat(BLOCK_INDENT));
        block.push_str(line);
        block.push('\n');
    }
    Ok(block)
}

fn first_content_line_is_indented(state: &str) -> bool {
    state
        .lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.starts_with(' ') || line.starts_with('\t'))
        .unwrap_or(false)
}

fn push_indented(output: &mut String, yaml: &str) {
    let indent = " ".repeat(BLOCK_INDENT);
    for line in yaml.lines() {
        if line.is_empty() {
            output.push('\n');
            continue;
        }
        output.push_str(&indent);
        output.push_str(line);
        output.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wm_engine::{canonical_questions, parse_record};

    const PROSE: &str = "## Context\n\nChose SQLite over Postgres.\n\nSee @wiki/reference/bm25-search.";

    #[test]
    fn round_trips_each_record_bearing_type() {
        for page_type in [
            PageType::Decision,
            PageType::Pattern,
            PageType::Concept,
            PageType::Howto,
            PageType::Reference,
        ] {
            let body = build_record_body(&page_type, PROSE).expect("body should render");
            assert!(body.starts_with("schema_version: 1\nstate: |-\n"));
            let record = parse_record(&page_type, &body).expect("body should parse");
            assert_eq!(record.state, PROSE);
            assert_eq!(record.questions, canonical_questions(&page_type));
            assert!(record.answers.is_empty());
        }
    }

    #[test]
    fn preserves_relative_indentation_in_state() {
        let state = "## Steps\n\n1. one\n\n    code block\n    second line";
        let body = build_record_body(&PageType::Howto, state).expect("body should render");
        let record = parse_record(&PageType::Howto, &body).expect("body should parse");
        assert_eq!(record.state, state);
    }

    #[test]
    fn uses_explicit_indent_when_first_line_is_indented() {
        let state = "    indented first\nback to normal";
        let body = build_record_body(&PageType::Concept, state).expect("body should render");
        assert!(body.contains("state: |2-\n"));
        let record = parse_record(&PageType::Concept, &body).expect("body should parse");
        assert_eq!(record.state, state);
    }

    #[test]
    fn preserves_blank_lines_in_state() {
        let state = "a\n\nb";
        let body = build_record_body(&PageType::Concept, state).expect("body should render");
        let record = parse_record(&PageType::Concept, &body).expect("body should parse");
        assert_eq!(record.state, state);
    }
}
