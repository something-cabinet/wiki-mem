use serde::Serialize;

/// A hard failure that prevented a page from being converted or classified.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParseFailure {
    /// Wiki-relative path of the offending page.
    pub path: String,
    /// Error text produced by the parser.
    pub error: String,
}
