use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParseFailure {
    pub path: String,
    pub error: String,
}
