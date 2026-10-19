use std::collections::HashMap;
use std::sync::RwLock;

use crate::engine::SourceEntry;
use crate::error::{ToolError, ToolResult};

#[derive(Default)]
pub struct SourceRegistryState {
    registry: RwLock<HashMap<String, SourceEntry>>,
}

impl SourceRegistryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, entry: SourceEntry) -> ToolResult<()> {
        let id = entry.id.clone();
        self.registry
            .write()
            .map_err(|_| ToolError::lock_poisoned("registry"))?
            .insert(id, entry);
        Ok(())
    }

    pub fn update<T>(
        &self,
        id: &str,
        change: impl FnOnce(&mut SourceEntry) -> ToolResult<T>,
    ) -> ToolResult<T> {
        let mut registry = self
            .registry
            .write()
            .map_err(|_| ToolError::lock_poisoned("registry"))?;
        let entry = registry
            .get_mut(id)
            .ok_or_else(|| ToolError::not_found("source", id))?;
        change(entry)
    }

    pub fn update_if_present<T>(
        &self,
        id: &str,
        change: impl FnOnce(&mut SourceEntry) -> T,
    ) -> ToolResult<Option<T>> {
        let mut registry = self
            .registry
            .write()
            .map_err(|_| ToolError::lock_poisoned("registry"))?;
        Ok(registry.get_mut(id).map(change))
    }

    pub fn get(&self, id: &str) -> ToolResult<SourceEntry> {
        let registry = self
            .registry
            .read()
            .map_err(|_| ToolError::lock_poisoned("registry"))?;
        registry
            .get(id)
            .cloned()
            .ok_or_else(|| ToolError::not_found("source", id))
    }

    pub fn entries(&self) -> ToolResult<Vec<SourceEntry>> {
        let registry = self
            .registry
            .read()
            .map_err(|_| ToolError::lock_poisoned("registry"))?;
        Ok(registry.values().cloned().collect())
    }

    pub fn is_tracked(&self, content_hash: &str, original_path: &str) -> ToolResult<bool> {
        let registry = self
            .registry
            .read()
            .map_err(|_| ToolError::lock_poisoned("registry"))?;
        Ok(registry.values().any(|entry| {
            entry.content_hash == content_hash
                || entry.original_path.as_deref() == Some(original_path)
        }))
    }

    pub fn remove(&self, id: &str) -> ToolResult<()> {
        self.registry
            .write()
            .map_err(|_| ToolError::lock_poisoned("registry"))?
            .remove(id)
            .ok_or_else(|| ToolError::not_found("source", id))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRY_FIELDS: [&str; 1] = ["registry"];

    fn sample_entry(id: &str) -> SourceEntry {
        SourceEntry {
            id: id.to_string(),
            original_path: Some(format!("/tmp/{id}.rs")),
            stored_path: std::path::PathBuf::from(format!("/tmp/stored-{id}.rs")),
            content_hash: format!("hash-{id}"),
            state: crate::engine::SourceState::Pending,
            added_at: "2026-01-01T00:00:00Z".to_string(),
            last_processed_at: None,
            page_refs: Vec::new(),
            page_count: 0,
            error_message: None,
            retry_count: 0,
        }
    }

    fn state_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("source_registry_state_model.rs");
        let start = SOURCE
            .find("pub struct SourceRegistryState {")
            .expect("SourceRegistryState declaration exists");
        let body = &SOURCE[start..];
        let end = body
            .find("\n}")
            .expect("SourceRegistryState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = state_struct_body()
            .lines()
            .map(str::trim)
            .filter(|line| {
                needles
                    .iter()
                    .any(|needle| line.starts_with(needle.as_str()))
            })
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one declaration of {field}"
        );
        matches[0]
    }

    #[test]
    fn test_source_registry_state_declares_its_field_private() {
        for field in REGISTRY_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
    }

    #[test]
    fn test_handle_owns_inserted_entries() {
        let sources = SourceRegistryState::new();
        sources.insert(sample_entry("src_a")).expect("insert");

        assert_eq!(sources.entries().expect("entries").len(), 1);
        assert_eq!(sources.get("src_a").expect("get").id, "src_a");
    }

    #[test]
    fn test_update_mutates_the_owned_entry() {
        let sources = SourceRegistryState::new();
        sources.insert(sample_entry("src_a")).expect("insert");

        sources
            .update("src_a", |entry| {
                entry.retry_count = entry.retry_count.wrapping_add(1);
                entry.state = crate::engine::SourceState::Processing;
                Ok(())
            })
            .expect("update");

        let entry = sources.get("src_a").expect("get");
        assert_eq!(entry.retry_count, 1);
        assert_eq!(entry.state, crate::engine::SourceState::Processing);
    }

    #[test]
    fn test_separate_handles_do_not_share_state() {
        let first = SourceRegistryState::new();
        let second = SourceRegistryState::new();
        first.insert(sample_entry("src_a")).expect("insert");

        assert_eq!(first.entries().expect("entries").len(), 1);
        assert!(second.entries().expect("entries").is_empty());
    }

    #[test]
    fn test_missing_entry_is_reported_as_not_found() {
        let sources = SourceRegistryState::new();

        assert_eq!(
            sources.get("src_missing").expect_err("get must fail").code,
            "NOT_FOUND"
        );
        assert_eq!(
            sources
                .update("src_missing", |_| Ok(()))
                .expect_err("update must fail")
                .code,
            "NOT_FOUND"
        );
        assert_eq!(
            sources
                .remove("src_missing")
                .expect_err("remove must fail")
                .code,
            "NOT_FOUND"
        );
    }

    #[test]
    fn test_is_tracked_matches_hash_or_original_path() {
        let sources = SourceRegistryState::new();
        sources.insert(sample_entry("src_a")).expect("insert");

        assert!(sources
            .is_tracked("hash-src_a", "/elsewhere.rs")
            .expect("tracked by hash"));
        assert!(sources
            .is_tracked("other", "/tmp/src_a.rs")
            .expect("tracked by path"));
        assert!(!sources
            .is_tracked("other", "/tmp/other.rs")
            .expect("not tracked"));
    }

    #[test]
    fn test_remove_drops_the_entry() {
        let sources = SourceRegistryState::new();
        sources.insert(sample_entry("src_a")).expect("insert");

        sources.remove("src_a").expect("remove");
        assert!(sources.entries().expect("entries").is_empty());
    }
}
