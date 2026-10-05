use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MigrationNote {
    pub path: String,
    pub note: String,
}
