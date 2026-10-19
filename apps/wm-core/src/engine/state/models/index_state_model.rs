use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::{ArcSwap, Guard};
use wm_embed::{Embedder, VectorStore};

use crate::engine::SectionDoc;
use crate::search::Bm25Index;

pub struct IndexState {
    section_corpus: ArcSwap<Vec<SectionDoc>>,
    bm25_index: ArcSwap<Bm25Index>,
    vector_registry: ArcSwap<HashMap<String, Vec<f32>>>,
    embedder: Box<dyn Embedder + Send + Sync>,
    vector_store: VectorStore,
}

impl IndexState {
    pub fn new(embedder: Box<dyn Embedder + Send + Sync>, vector_store: VectorStore) -> Self {
        Self {
            section_corpus: ArcSwap::new(Arc::new(Vec::new())),
            bm25_index: ArcSwap::new(Arc::new(Bm25Index::new())),
            vector_registry: ArcSwap::new(Arc::new(HashMap::new())),
            embedder,
            vector_store,
        }
    }

    pub fn section_corpus(&self) -> Guard<Arc<Vec<SectionDoc>>> {
        self.section_corpus.load()
    }

    pub fn replace_section_corpus(&self, sections: Arc<Vec<SectionDoc>>) {
        self.section_corpus.store(sections);
    }

    pub fn update_section_corpus<F>(&self, update: F)
    where
        F: Fn(&[SectionDoc]) -> Vec<SectionDoc>,
    {
        self.section_corpus
            .rcu(|old| Arc::new(update(old.as_slice())));
    }

    pub fn bm25_index(&self) -> Guard<Arc<Bm25Index>> {
        self.bm25_index.load()
    }

    pub fn replace_bm25_index(&self, index: Arc<Bm25Index>) {
        self.bm25_index.store(index);
    }

    pub fn vector_registry(&self) -> Guard<Arc<HashMap<String, Vec<f32>>>> {
        self.vector_registry.load()
    }

    pub fn replace_vector_registry(&self, registry: Arc<HashMap<String, Vec<f32>>>) {
        self.vector_registry.store(registry);
    }

    pub fn embedder(&self) -> &(dyn Embedder + Send + Sync) {
        self.embedder.as_ref()
    }

    pub fn vector_store(&self) -> &VectorStore {
        &self.vector_store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wm_embed::NoopEmbedder;

    const INDEX_FIELDS: [&str; 5] = [
        "section_corpus",
        "bm25_index",
        "vector_registry",
        "embedder",
        "vector_store",
    ];
    const ARCSWAP_FIELDS: [&str; 3] = ["section_corpus", "bm25_index", "vector_registry"];

    fn sample_section(id: &str) -> SectionDoc {
        SectionDoc {
            section_id: id.to_string(),
            page_id: "wiki:concepts:demo".to_string(),
            header: "Body".to_string(),
            body: "alpha beta gamma".to_string(),
            title: "Demo".to_string(),
            tags: vec!["demo".to_string()],
        }
    }

    fn sample_doc(section: &SectionDoc) -> crate::search::IndexedDoc {
        crate::search::IndexedDoc {
            id: section.section_id.clone(),
            fields: vec![crate::search::Field::new("body", &section.body, 1.0)],
        }
    }

    fn handle(project_root: &std::path::Path) -> IndexState {
        IndexState::new(
            Box::new(NoopEmbedder::new()),
            VectorStore::new("none", project_root),
        )
    }

    fn index_struct_body() -> &'static str {
        const SOURCE: &str = include_str!("index_state_model.rs");
        let start = SOURCE
            .find("pub struct IndexState {")
            .expect("IndexState declaration exists");
        let body = &SOURCE[start..];
        let end = body.find("\n}").expect("IndexState body terminates");
        &body[..end]
    }

    fn declared_field_line(field: &str) -> &'static str {
        let needles = [format!("{field}:"), format!("pub {field}:")];
        let matches: Vec<&str> = index_struct_body()
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
    fn test_index_state_declares_its_five_fields_private() {
        for field in INDEX_FIELDS {
            let declaration = declared_field_line(field);
            assert!(
                !declaration.starts_with("pub"),
                "{field} must stay private so no sibling handle can read it"
            );
        }
    }

    #[test]
    fn test_read_path_keeps_arcswap_and_adds_no_lock() {
        let body = index_struct_body();

        for field in ARCSWAP_FIELDS {
            assert!(
                declared_field_line(field).starts_with(&format!("{field}: ArcSwap<")),
                "{field} must stay an ArcSwap snapshot"
            );
        }
        assert!(!body.contains("Mutex"), "no Mutex may wrap the index state");
        assert!(
            !body.contains("RwLock"),
            "no RwLock may wrap the index state"
        );
    }

    #[test]
    fn test_reads_return_arcswap_guards() {
        let dir = tempfile::tempdir().expect("temp dir");
        let index = handle(dir.path());

        let corpus: Guard<Arc<Vec<SectionDoc>>> = index.section_corpus();
        let bm25: Guard<Arc<Bm25Index>> = index.bm25_index();
        let registry: Guard<Arc<HashMap<String, Vec<f32>>>> = index.vector_registry();

        assert!(corpus.is_empty());
        assert_eq!(bm25.total_docs, 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_handle_owns_written_corpus_and_index() {
        let dir = tempfile::tempdir().expect("temp dir");
        let index = handle(dir.path());
        let section = sample_section("wiki:concepts:demo#body");

        index.replace_section_corpus(Arc::new(vec![section.clone()]));
        index.replace_bm25_index(Arc::new(Bm25Index::build(vec![sample_doc(&section)])));
        index.replace_vector_registry(Arc::new(HashMap::from([(
            section.section_id.clone(),
            vec![0.5_f32],
        )])));

        assert_eq!(index.section_corpus().len(), 1);
        assert_eq!(index.bm25_index().total_docs, 1);
        assert_eq!(
            index
                .vector_registry()
                .get("wiki:concepts:demo#body")
                .expect("registered vector")
                .len(),
            1
        );
    }

    #[test]
    fn test_update_section_corpus_replaces_the_owned_snapshot() {
        let dir = tempfile::tempdir().expect("temp dir");
        let index = handle(dir.path());
        index.replace_section_corpus(Arc::new(vec![sample_section("a"), sample_section("b")]));

        index.update_section_corpus(|sections| {
            sections
                .iter()
                .filter(|section| section.section_id != "a")
                .cloned()
                .collect()
        });

        assert_eq!(index.section_corpus().len(), 1);
        assert_eq!(index.section_corpus()[0].section_id, "b");
    }

    #[test]
    fn test_separate_handles_do_not_share_state() {
        let dir = tempfile::tempdir().expect("temp dir");
        let first = handle(dir.path());
        let second = handle(dir.path());
        first.replace_section_corpus(Arc::new(vec![sample_section("a")]));

        assert_eq!(first.section_corpus().len(), 1);
        assert!(second.section_corpus().is_empty());
    }

    #[test]
    fn test_embedder_and_vector_store_access_are_borrows() {
        let dir = tempfile::tempdir().expect("temp dir");
        let index = handle(dir.path());

        assert!(!index.embedder().is_loaded());
        assert_eq!(index.embedder().model_name(), "none");
        assert!(index.vector_store().snapshot().is_empty());
    }
}
