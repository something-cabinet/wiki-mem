use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    Auto,
    Keyword,
    Semantic,
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::Hybrid
    }
}

impl std::str::FromStr for SearchMode {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "auto" => SearchMode::Auto,
            "semantic" => SearchMode::Semantic,
            "hybrid" => SearchMode::Hybrid,
            _ => SearchMode::Keyword,
        })
    }
}

impl SearchMode {
    /// Named `from_str` rather than implementing `FromStr` because this
    /// method accepts a looser format (lowercase, partial matches) than
    /// a strict FromStr impl would warrant.
    pub fn from_str(s: &str) -> Self {
        s.parse().expect("SearchMode::from_str is infallible")
    }

    pub fn auto_detect(query: &str) -> Self {
        let has_code_pattern = query.contains('_')
            || query.contains('-')
            || query.chars().filter(|c| c.is_uppercase()).count() > 2;
        if has_code_pattern {
            SearchMode::Keyword
        } else {
            SearchMode::Hybrid
        }
    }
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchMode::Auto => write!(f, "auto"),
            SearchMode::Keyword => write!(f, "keyword"),
            SearchMode::Semantic => write!(f, "semantic"),
            SearchMode::Hybrid => write!(f, "hybrid"),
        }
    }
}
