use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecisionError {
    #[error("page type '{0}' does not carry a typed record")]
    NotRecordBearing(String),
    #[error("record YAML is invalid: {0}")]
    InvalidYaml(String),
    #[error("record must be a YAML mapping")]
    NotAMapping,
    #[error("unknown top-level key '{0}' in record")]
    UnknownTopLevelKey(String),
    #[error("record is missing required key '{0}'")]
    MissingField(String),
    #[error("unsupported schema_version {0}; expected 1")]
    UnsupportedSchemaVersion(u32),
    #[error("record 'state' must be non-empty")]
    EmptyState,
    #[error("record uses a YAML anchor or alias, which is not allowed")]
    AnchorOrAlias,
    #[error("record contains a YAML document separator, which is not allowed")]
    DocumentSeparator,
    #[error("record must declare 1-32 questions, found {count}")]
    InvalidQuestionCount { count: usize },
    #[error("duplicate question id '{0}'")]
    DuplicateQuestionId(String),
    #[error("question id '{0}' does not match ^[a-z][a-z0-9_]{{0,47}}$")]
    InvalidQuestionId(String),
    #[error("question '{id}' instructions must be a single line of 1-240 characters")]
    InvalidInstructions { id: String },
    #[error("choice question '{id}' must declare 2-20 options")]
    InvalidOptionCount { id: String },
    #[error("choice question '{id}' has duplicate option '{option}'")]
    DuplicateOption { id: String, option: String },
    #[error("score question '{id}' must declare 2-10 levels")]
    InvalidLevelCount { id: String },
    #[error("score question '{id}' has duplicate level '{level}'")]
    DuplicateLevel { id: String, level: String },
    #[error("question '{0}' sets 'multi' but is not a choice question")]
    MultiOnNonChoice(String),
    #[error("answer key '{0}' does not match any question id")]
    UnknownAnswerKey(String),
    #[error("answer for question '{id}' has an invalid value")]
    InvalidAnswerValue { id: String },
    #[error("record questions do not match the canonical set for page type '{0}'")]
    CanonicalMismatch(String),
    #[error("decision backend failed: {detail}")]
    Backend { detail: String },
    #[error("model manifest is invalid: {0}")]
    InvalidManifest(String),
    #[error("backend returned no result for task '{0}'")]
    MissingTaskResult(String),
    #[error("backend labels/probabilities for task '{task}' do not match the spec")]
    ProbabilityMismatch { task: String },
    #[error("record has {questions} questions but {specs} specs were supplied")]
    SpecCountMismatch { questions: usize, specs: usize },
}
