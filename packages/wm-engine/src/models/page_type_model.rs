use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::status::PageStatus;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PageType {
    Task,
    Spec,
    Concept,
    Pattern,
    Decision,
    Core,
    Memory,
    Howto,
    Reference,
    #[serde(rename = "note")]
    Note,
    Rule,
}

impl PageType {
    /// Returns the singular type name (e.g., "task", "spec", "core").
    pub fn as_str(&self) -> &'static str {
        match self {
            PageType::Task => "task",
            PageType::Spec => "spec",
            PageType::Concept => "concept",
            PageType::Pattern => "pattern",
            PageType::Decision => "decision",
            PageType::Core => "core",
            PageType::Memory => "memory",
            PageType::Howto => "howto",
            PageType::Reference => "reference",
            PageType::Note => "note",
            PageType::Rule => "rule",
        }
    }

    /// Returns the plural directory name (e.g., "tasks", "specs", "core").
    pub fn dir_name(&self) -> &'static str {
        match self {
            PageType::Memory => "memory",
            PageType::Note => "notes",
            PageType::Reference => "reference",
            PageType::Howto => "howto",
            _ => {
                let s = self.as_str();
                match self {
                    PageType::Core | PageType::Rule => s,
                    _ => {
                        match self {
                            PageType::Task => "tasks",
                            PageType::Spec => "specs",
                            PageType::Concept => "concepts",
                            PageType::Pattern => "patterns",
                            PageType::Decision => "decisions",
                            _ => s,
                        }
                    }
                }
            }
        }
    }

    /// Parse from a singular type name (e.g., "task" → Some(PageType::Task)).
    /// Returns None for unknown type names (caller decides fallback behavior).
    pub fn from_type_name(s: &str) -> Option<PageType> {
        match s {
            "task" => Some(PageType::Task),
            "spec" => Some(PageType::Spec),
            "concept" => Some(PageType::Concept),
            "pattern" => Some(PageType::Pattern),
            "decision" => Some(PageType::Decision),
            "memory" => Some(PageType::Memory),
            "howto" | "guide" => Some(PageType::Howto),
            "reference" => Some(PageType::Reference),
            "note" | "notes" => Some(PageType::Note),
            "rule" => Some(PageType::Rule),
            "core" => Some(PageType::Core),
            _ => None,
        }
    }

    /// Parse from a plural directory name (e.g., "tasks" → Some(PageType::Task)).
    pub fn from_dir_name(dir: &str) -> Option<PageType> {
        match dir {
            "tasks" => Some(PageType::Task),
            "specs" => Some(PageType::Spec),
            "concepts" => Some(PageType::Concept),
            "patterns" => Some(PageType::Pattern),
            "decisions" => Some(PageType::Decision),
            "memory" => Some(PageType::Memory),
            "howto" => Some(PageType::Howto),
            "reference" => Some(PageType::Reference),
            "notes" => Some(PageType::Note),
            "rules" => Some(PageType::Rule),
            "core" => Some(PageType::Core),
            _ => None,
        }
    }

    /// All page type names as singular strings (for `page_types_available` etc.).
    pub fn all_type_names() -> &'static [&'static str] {
        &[
            "task",
            "spec",
            "concept",
            "pattern",
            "decision",
            "howto",
            "reference",
            "core",
            "rule",
            "memory",
            "note",
        ]
    }

    /// All page type directory names as plural strings (for validation, reference).
    pub fn all_dir_names() -> &'static [&'static str] {
        &[
            "tasks",
            "specs",
            "concepts",
            "patterns",
            "decisions",
            "howto",
            "reference",
            "core",
            "rules",
            "memory",
            "notes",
        ]
    }

    pub fn allowed_statuses(&self) -> &[PageStatus] {
        use PageStatus::*;
        match self {
            PageType::Task => &[Todo, InProgress, InReview, Done, Blocked, Cancelled],
            PageType::Spec => &[Draft, Reviewed, Approved, Superseded],
            PageType::Decision => &[Draft, Approved, Superseded, Rejected, Archived],
            PageType::Rule => &[Draft, Active, Superseded, Archived],
            _ => &[Draft, Reviewed, Approved, Archived],
        }
    }

    pub fn priority_rank(&self) -> u8 {
        match self {
            PageType::Task => 7,
            PageType::Spec => 6,
            PageType::Pattern => 5,
            PageType::Concept => 4,
            PageType::Decision => 3,
            PageType::Core => 9,
            PageType::Memory => 2,
            PageType::Howto => 2,
            PageType::Reference => 1,
            PageType::Note => 0,
            PageType::Rule => 8,
        }
    }
}
