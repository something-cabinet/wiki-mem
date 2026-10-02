use serde::Serialize;

/// A non-fatal observation attached to a page during planning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MigrationNote {
    /// Wiki-relative path of the page the note belongs to.
    pub path: String,
    /// Human-readable note text.
    pub note: String,
}
