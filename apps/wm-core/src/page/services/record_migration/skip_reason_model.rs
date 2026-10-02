use serde::Serialize;

/// Why a record-bearing page was left untouched by the migration planner.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "reason")]
pub enum SkipReason {
    /// The file has no leading `---` frontmatter block.
    MissingFrontmatter,
    /// A leading frontmatter block is not a YAML mapping.
    UnparseableFrontmatter { block: usize },
    /// Two frontmatter blocks disagree on a top-level value.
    DuplicateFrontmatterConflict { key: String },
    /// The normalized body is empty, so `state` would be empty.
    EmptyState,
    /// The generated record body failed to parse back.
    InvalidRecordBody { error: String },
    /// The configured conversion limit was already reached.
    LimitReached,
}

impl SkipReason {
    /// Coarse bucket used to aggregate skip counts in the report.
    pub fn kind(&self) -> &'static str {
        match self {
            SkipReason::MissingFrontmatter => "missing-frontmatter",
            SkipReason::UnparseableFrontmatter { .. } => "unparseable-frontmatter",
            SkipReason::DuplicateFrontmatterConflict { .. } => "duplicate-frontmatter-conflict",
            SkipReason::InvalidRecordBody { .. } => "invalid-record-body",
            SkipReason::EmptyState => "empty-state",
            SkipReason::LimitReached => "limit-reached",
        }
    }

    /// Per-page detail kept next to the aggregate kind.
    pub fn detail(&self) -> String {
        match self {
            SkipReason::MissingFrontmatter => "no leading frontmatter block".to_owned(),
            SkipReason::UnparseableFrontmatter { block } => {
                format!("frontmatter block {block} is not a mapping")
            }
            SkipReason::DuplicateFrontmatterConflict { key } => {
                format!("duplicate frontmatter disagrees on `{key}`")
            }
            SkipReason::InvalidRecordBody { error } => {
                format!("generated record body did not parse: {error}")
            }
            SkipReason::EmptyState => "normalized body is empty".to_owned(),
            SkipReason::LimitReached => "conversion limit already reached".to_owned(),
        }
    }
}
