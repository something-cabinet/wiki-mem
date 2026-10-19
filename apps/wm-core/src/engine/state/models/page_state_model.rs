use std::collections::HashMap;
use std::sync::RwLock;

use dashmap::DashMap;

use crate::engine::WikiPageContent;
use crate::error::{ToolError, ToolResult};

#[derive(Default)]
pub struct PageState {
    page_contents: DashMap<String, WikiPageContent>,
    content_hashes: RwLock<HashMap<String, String>>,
}

impl PageState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_content(&self, id: &str, content: WikiPageContent) {
        self.page_contents.insert(id.to_string(), content);
    }

    pub fn with_content<T>(&self, id: &str, read: impl FnOnce(&WikiPageContent) -> T) -> Option<T> {
        self.page_contents.get(id).map(|entry| read(entry.value()))
    }

    pub fn contains_content(&self, id: &str) -> bool {
        self.page_contents.contains_key(id)
    }

    pub fn content_count(&self) -> usize {
        self.page_contents.len()
    }

    pub fn remove_content(&self, id: &str) -> bool {
        self.page_contents.remove(id).is_some()
    }

    pub fn content_hash(&self, id: &str) -> ToolResult<Option<String>> {
        let hashes = self
            .content_hashes
            .read()
            .map_err(|_| ToolError::lock_poisoned("content_hashes"))?;
        Ok(hashes.get(id).cloned())
    }

    pub fn set_content_hash(&self, id: &str, hash: &str) -> ToolResult<()> {
        self.content_hashes
            .write()
            .map_err(|_| ToolError::lock_poisoned("content_hashes"))?
            .insert(id.to_string(), hash.to_string());
        Ok(())
    }

    pub fn remove_content_hash(&self, id: &str) -> ToolResult<bool> {
        let mut hashes = self
            .content_hashes
            .write()
            .map_err(|_| ToolError::lock_poisoned("content_hashes"))?;
        Ok(hashes.remove(id).is_some())
    }

    pub fn hash_count(&self) -> ToolResult<usize> {
        let hashes = self
            .content_hashes
            .read()
            .map_err(|_| ToolError::lock_poisoned("content_hashes"))?;
        Ok(hashes.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SectionDoc;

    const PAGE_FIELDS: [&str; 2] = ["page_contents", "content_hashes"];

    fn sample_content(id: &str) -> WikiPageContent {
        WikiPageContent {
            raw: format!("# {id}\n\nbody"),
            sections: vec![SectionDoc {
                section_id: format!("{id}#body"),
                page_id: id.to_string(),
                header: "body".to_string(),
                body: "body".to_string(),
                title: id.to_string(),
                tags: Vec::new(),
            }],
            meta: None,
        }
    }

    fn page_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("page_state_model.rs");
        let start = SOURCE
            .find("pub struct PageState {")
            .expect("PageState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("PageState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = page_struct_body()
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
    fn test_page_state_declares_only_its_two_fields_private() {
        for field in PAGE_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
        assert_eq!(
            page_struct_body()
                .lines()
                .map(str::trim)
                .filter(|line| line.ends_with(','))
                .count(),
            PAGE_FIELDS.len(),
            "PageState may not carry a sibling handle field"
        );
    }

    #[test]
    fn test_content_hashes_stays_behind_the_original_rwlock() {
        assert!(
            declared_field_line("content_hashes")
                .starts_with("content_hashes: RwLock<HashMap<String, String>>"),
            "content_hashes must keep its RwLock<HashMap<String, String>> semantics"
        );
    }

    #[test]
    fn test_handle_owns_page_contents() {
        let pages = PageState::new();
        pages.insert_content("wiki:concepts:demo", sample_content("wiki:concepts:demo"));

        assert_eq!(pages.content_count(), 1);
        assert!(pages.contains_content("wiki:concepts:demo"));
        let section_id = pages
            .with_content("wiki:concepts:demo", |content| {
                content.sections[0].section_id.clone()
            })
            .expect("owned content");
        assert_eq!(section_id, "wiki:concepts:demo#body");

        assert!(pages.remove_content("wiki:concepts:demo"));
        assert_eq!(pages.content_count(), 0);
    }

    #[test]
    fn test_handle_owns_content_hashes() {
        let pages = PageState::new();
        pages
            .set_content_hash("wiki:concepts:demo", "hash-a")
            .expect("store hash");

        assert_eq!(
            pages.content_hash("wiki:concepts:demo").expect("read hash"),
            Some("hash-a".to_string())
        );
        assert_eq!(pages.hash_count().expect("count hashes"), 1);
        assert!(pages
            .remove_content_hash("wiki:concepts:demo")
            .expect("remove hash"));
        assert_eq!(pages.hash_count().expect("count hashes"), 0);
    }

    #[test]
    fn test_separate_handles_do_not_share_state() {
        let first = PageState::new();
        let second = PageState::new();
        first.insert_content("wiki:concepts:demo", sample_content("wiki:concepts:demo"));
        first
            .set_content_hash("wiki:concepts:demo", "hash-a")
            .expect("store hash");

        assert_eq!(first.content_count(), 1);
        assert_eq!(second.content_count(), 0);
        assert_eq!(
            second
                .content_hash("wiki:concepts:demo")
                .expect("read hash"),
            None
        );
    }
}
