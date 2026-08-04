use std::path::{Path, PathBuf};

use crate::error::ToolResult;
use crate::shared::helpers::path_confine_helper::{confine, confine_strict};

pub struct UserPath(String);

impl UserPath {
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    pub fn confine_under(&self, root: &Path) -> ToolResult<PathBuf> {
        confine(root, Path::new(&self.0))
    }

    pub fn confine_strict_under(&self, root: &Path) -> ToolResult<PathBuf> {
        confine_strict(root, Path::new(&self.0))
    }

    pub fn raw(&self) -> &str {
        &self.0
    }
}

impl From<String> for UserPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserPath {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confine_under_rejects_traversal() {
        let up = UserPath::new("../../etc/passwd");
        assert!(up.confine_under(Path::new(".wm/wiki")).is_err());
    }

    #[test]
    fn confine_under_allows_valid() {
        let up = UserPath::new("specs/x.md");
        let p = up.confine_under(Path::new(".wm/wiki")).expect("valid");
        assert_eq!(p, PathBuf::from(".wm/wiki/specs/x.md"));
    }

    #[test]
    fn strict_rejects_dotfiles() {
        let up = UserPath::new(".git/config");
        assert!(up.confine_strict_under(Path::new(".")).is_err());
    }

    #[test]
    fn raw_returns_original() {
        let up = UserPath::new("tasks/fix.md");
        assert_eq!(up.raw(), "tasks/fix.md");
    }
}
