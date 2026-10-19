use dashmap::DashMap;

use crate::engine::MemoryEntry;

#[derive(Default)]
pub struct MemoryState {
    entries: DashMap<String, MemoryEntry>,
}

impl MemoryState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn insert(&self, entry: MemoryEntry) {
        let id = entry.id.clone();
        self.entries.insert(id, entry);
    }

    pub fn snapshot(&self) -> Vec<MemoryEntry> {
        self.entries
            .iter()
            .map(|item| item.value().clone())
            .collect()
    }

    pub fn remove(&self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMORY_FIELDS: [&str; 1] = ["entries"];

    fn sample_entry(id: &str) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            title: format!("Memory {id}"),
            content: format!("content for {id}"),
            tags: vec!["session".to_string()],
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            status: None,
        }
    }

    fn memory_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("memory_state_model.rs");
        let start = SOURCE
            .find("pub struct MemoryState {")
            .expect("MemoryState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("MemoryState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = memory_struct_body()
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
    fn test_memory_state_declares_its_field_private() {
        for field in MEMORY_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
    }

    #[test]
    fn test_handle_owns_inserted_entries() {
        let memory = MemoryState::new();
        memory.insert(sample_entry("mem_a"));

        assert_eq!(memory.count(), 1);
        let entries = memory.snapshot();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "mem_a");
    }

    #[test]
    fn test_snapshot_copies_entries_out_of_the_store() {
        let memory = MemoryState::new();
        memory.insert(sample_entry("mem_a"));

        let mut entries = memory.snapshot();
        entries[0].title = "edited".to_string();

        assert_eq!(memory.snapshot()[0].title, "Memory mem_a");
    }

    #[test]
    fn test_separate_handles_do_not_share_state() {
        let first = MemoryState::new();
        let second = MemoryState::new();
        first.insert(sample_entry("mem_a"));

        assert_eq!(first.count(), 1);
        assert_eq!(second.count(), 0);
    }

    #[test]
    fn test_insert_replaces_by_the_entry_id() {
        let memory = MemoryState::new();
        memory.insert(sample_entry("mem_a"));
        let mut replacement = sample_entry("mem_a");
        replacement.title = "Replacement".to_string();
        memory.insert(replacement);

        assert_eq!(memory.count(), 1);
        assert_eq!(memory.snapshot()[0].title, "Replacement");
    }

    #[test]
    fn test_remove_drops_the_entry() {
        let memory = MemoryState::new();
        memory.insert(sample_entry("mem_a"));

        assert!(memory.remove("mem_a"));
        assert_eq!(memory.count(), 0);
        assert!(!memory.remove("mem_a"));
    }
}
