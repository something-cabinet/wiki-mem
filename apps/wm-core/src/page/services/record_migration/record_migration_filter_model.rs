/// Restricts which pages a migration plan may touch.
///
/// An empty `only` selects every record-bearing page. `limit` caps the number
/// of pages that would be converted; already-record no-ops do not consume it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordMigrationFilter {
    only: Vec<String>,
    limit: Option<usize>,
}

impl RecordMigrationFilter {
    /// Builds a filter from raw CLI values, normalizing `only` paths.
    pub fn new(only: Vec<String>, limit: Option<usize>) -> Self {
        Self {
            only: only.into_iter().map(|path| normalize_only_path(&path)).collect(),
            limit,
        }
    }

    /// Whether a wiki-relative `path` is eligible to be planned at all.
    pub fn accepts(&self, path: &str) -> bool {
        self.only.is_empty() || self.only.iter().any(|only| only == path)
    }

    /// Whether `converted` pages already consume the configured limit.
    pub fn limit_reached(&self, converted: usize) -> bool {
        self.limit.is_some_and(|limit| converted >= limit)
    }

    /// Whether the filter leaves the full wiki in scope.
    pub fn is_unrestricted(&self) -> bool {
        self.only.is_empty() && self.limit.is_none()
    }
}

fn normalize_only_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_accepts_every_path() {
        let filter = RecordMigrationFilter::default();
        assert!(filter.accepts("decisions/one.md"));
        assert!(filter.is_unrestricted());
    }

    #[test]
    fn only_filter_matches_exact_relative_path() {
        let filter = RecordMigrationFilter::new(vec!["decisions/one.md".to_owned()], None);
        assert!(filter.accepts("decisions/one.md"));
        assert!(!filter.accepts("decisions/two.md"));
        assert!(!filter.is_unrestricted());
    }

    #[test]
    fn only_filter_normalizes_separators_and_prefixes() {
        let filter = RecordMigrationFilter::new(vec![" .\\decisions\\one.md ".to_owned()], None);
        assert!(filter.accepts("decisions/one.md"));
    }

    #[test]
    fn limit_reports_reached_at_threshold() {
        let filter = RecordMigrationFilter::new(Vec::new(), Some(2));
        assert!(!filter.limit_reached(1));
        assert!(filter.limit_reached(2));
        assert!(filter.limit_reached(3));
        assert!(!filter.is_unrestricted());
    }
}
