use crate::models::PageType;

use crate::models::{QType, Question};

const OUTCOME_OPTIONS: &[&str] = &["adopted", "rejected", "deferred", "superseded", "abandoned"];
const CONFIDENCE_LEVELS: &[&str] = &["low", "medium", "high"];
const IMPACT_OPTIONS: &[&str] = &["local", "component", "system", "project-wide"];

const ENFORCEMENT_OPTIONS: &[&str] = &[
    "ci-enforced",
    "tool-enforced",
    "review-enforced",
    "convention-only",
];
const SEVERITY_LEVELS: &[&str] = &["advisory", "recommended", "required", "blocking"];
const APPLIES_TO_OPTIONS: &[&str] = &[
    "code",
    "tests",
    "docs",
    "configuration",
    "workflow",
    "security",
];

const PROBLEM_KIND_OPTIONS: &[&str] = &[
    "architecture",
    "api-design",
    "data-model",
    "error-handling",
    "performance",
    "testing",
    "ui",
    "tooling",
    "workflow",
];
const COMPLEXITY_LEVELS: &[&str] = &["trivial", "simple", "moderate", "complex"];

const CONCEPT_KIND_OPTIONS: &[&str] = &[
    "concept",
    "failure-analysis",
    "research-report",
    "reference-note",
];
const CATEGORY_OPTIONS: &[&str] = &[
    "architecture",
    "search-retrieval",
    "graph",
    "parser-format",
    "mcp-tooling",
    "cli",
    "storage",
    "embeddings",
    "web-ui",
    "process",
];
const MATURITY_LEVELS: &[&str] = &["raw", "exploratory", "established", "stable"];

const TASK_KIND_OPTIONS: &[&str] = &[
    "setup",
    "development",
    "testing",
    "release",
    "debugging",
    "operations",
    "integration",
];
const DIFFICULTY_LEVELS: &[&str] = &["beginner", "intermediate", "advanced", "expert"];

const REFERENCE_KIND_OPTIONS: &[&str] = &[
    "api",
    "cli",
    "configuration",
    "error-catalog",
    "schema",
    "scoring",
];
const SURFACE_OPTIONS: &[&str] = &[
    "mcp-tool",
    "cli-command",
    "rust-api",
    "http-api",
    "config-file",
];
const STABILITY_LEVELS: &[&str] = &["unstable", "evolving", "stable", "frozen"];

fn choice(id: &str, instructions: &str, options: &[&str], multi: bool) -> Question {
    Question {
        id: id.to_owned(),
        qtype: QType::Choice,
        instructions: instructions.to_owned(),
        multi,
        options: options.iter().map(|option| (*option).to_owned()).collect(),
        levels: Vec::new(),
    }
}

fn score(id: &str, instructions: &str, levels: &[&str]) -> Question {
    Question {
        id: id.to_owned(),
        qtype: QType::Score,
        instructions: instructions.to_owned(),
        multi: false,
        options: Vec::new(),
        levels: levels.iter().map(|level| (*level).to_owned()).collect(),
    }
}

fn noul(id: &str, instructions: &str) -> Question {
    Question {
        id: id.to_owned(),
        qtype: QType::Noul,
        instructions: instructions.to_owned(),
        multi: false,
        options: Vec::new(),
        levels: Vec::new(),
    }
}

fn decision_questions() -> Vec<Question> {
    vec![
        choice(
            "outcome",
            "What is the recorded outcome of this decision?",
            OUTCOME_OPTIONS,
            false,
        ),
        noul(
            "reversibility",
            "The decision can be reversed cheaply without data migration or cross-module breakage.",
        ),
        score(
            "confidence",
            "How strong is the recorded justification for the selected outcome?",
            CONFIDENCE_LEVELS,
        ),
        choice(
            "impact",
            "How wide is the blast radius of this decision?",
            IMPACT_OPTIONS,
            false,
        ),
    ]
}

fn rule_questions() -> Vec<Question> {
    vec![
        choice(
            "enforcement",
            "How is this rule enforced?",
            ENFORCEMENT_OPTIONS,
            false,
        ),
        score(
            "severity",
            "How severe is a violation of this rule?",
            SEVERITY_LEVELS,
        ),
        noul("has_exception", "This rule has documented exceptions."),
        choice(
            "applies_to",
            "Which surfaces does this rule apply to?",
            APPLIES_TO_OPTIONS,
            true,
        ),
    ]
}

fn pattern_questions() -> Vec<Question> {
    vec![
        choice(
            "problem_kind",
            "What kind of problem does this pattern solve?",
            PROBLEM_KIND_OPTIONS,
            false,
        ),
        noul(
            "preconditions_required",
            "This pattern requires specific preconditions to be met.",
        ),
        score(
            "complexity",
            "How complex is applying this pattern?",
            COMPLEXITY_LEVELS,
        ),
        noul(
            "language_specific",
            "This pattern is specific to a programming language.",
        ),
    ]
}

fn concept_questions() -> Vec<Question> {
    vec![
        choice(
            "kind",
            "What kind of concept document is this?",
            CONCEPT_KIND_OPTIONS,
            false,
        ),
        choice(
            "category",
            "Which domain category does this concept belong to?",
            CATEGORY_OPTIONS,
            false,
        ),
        score(
            "maturity",
            "How mature is the understanding of this concept?",
            MATURITY_LEVELS,
        ),
        noul("code_referenced", "This concept references concrete code."),
    ]
}

fn howto_questions() -> Vec<Question> {
    vec![
        choice(
            "task_kind",
            "What kind of task does this howto cover?",
            TASK_KIND_OPTIONS,
            false,
        ),
        noul(
            "prerequisites_complete",
            "All prerequisites for this howto are fully documented.",
        ),
        score(
            "difficulty",
            "What skill level does this howto require?",
            DIFFICULTY_LEVELS,
        ),
        noul(
            "has_verification",
            "This howto includes a verification step.",
        ),
    ]
}

fn reference_questions() -> Vec<Question> {
    vec![
        choice(
            "kind",
            "What kind of reference material is this?",
            REFERENCE_KIND_OPTIONS,
            false,
        ),
        choice(
            "surface",
            "Which surfaces does this reference document?",
            SURFACE_OPTIONS,
            true,
        ),
        score(
            "stability",
            "How stable is the documented surface?",
            STABILITY_LEVELS,
        ),
        noul("has_examples", "This reference includes usage examples."),
    ]
}

/// The canonical, order-sensitive question set for a record-bearing page type.
///
/// Single source of truth shared by the parser, validator, and migration. The
/// five convertible types (`decision`, `pattern`, `concept`, `howto`,
/// `reference`) return their fixed vocabularies; every other page type returns
/// an empty set and is not record-bearing.
pub fn canonical_questions(page_type: &PageType) -> Vec<Question> {
    match page_type {
        PageType::Decision => decision_questions(),
        PageType::Rule => rule_questions(),
        PageType::Pattern => pattern_questions(),
        PageType::Concept => concept_questions(),
        PageType::Howto => howto_questions(),
        PageType::Reference => reference_questions(),
        PageType::Task | PageType::Spec | PageType::Core | PageType::Memory | PageType::Note => {
            Vec::new()
        }
    }
}

pub fn is_record_bearing(page_type: &PageType) -> bool {
    match page_type {
        PageType::Decision
        | PageType::Pattern
        | PageType::Concept
        | PageType::Howto
        | PageType::Reference => true,
        PageType::Rule
        | PageType::Task
        | PageType::Spec
        | PageType::Core
        | PageType::Memory
        | PageType::Note => false,
    }
}
