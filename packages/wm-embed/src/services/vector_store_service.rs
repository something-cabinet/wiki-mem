use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use arc_swap::ArcSwap;

use wm_constants::*;

use crate::vector_db;

use crate::vector_db::EmbedVector;

pub struct VectorStore {
    pub entries: ArcSwap<HashMap<String, EmbedVector>>,
    pub model_name: String,
    pub hashes: ArcSwap<HashMap<String, [u8; 32]>>,
    pub db: Option<Arc<vector_db::VectorDb>>,
    pub embedding_metadata: std::sync::RwLock<crate::EmbeddingMetadata>,
    pub load_note: std::sync::RwLock<Option<String>>,
}

impl VectorStore {
    pub fn new(model_name: &str, project_root: &Path) -> Self {
        let db_dir = project_root.join(WM_DIR).join(STATE_DIR);
        let db_path = db_dir.join(VECTOR_DB_FILE);
        let _ = std::fs::create_dir_all(&db_dir);
        let db = vector_db::VectorDb::open(db_path, 0).ok().map(Arc::new);
        let metadata = crate::EmbeddingMetadata {
            model_name: model_name.to_string(),
            ..crate::EmbeddingMetadata::default()
        };
        Self {
            entries: ArcSwap::from_pointee(HashMap::new()),
            model_name: model_name.to_string(),
            hashes: ArcSwap::from_pointee(HashMap::new()),
            db,
            embedding_metadata: std::sync::RwLock::new(metadata),
            load_note: std::sync::RwLock::new(None),
        }
    }

    pub fn replace_entries_and_hashes(
        &self,
        new_entries: HashMap<String, EmbedVector>,
        new_hashes: HashMap<String, [u8; 32]>,
    ) {
        self.entries.store(Arc::new(new_entries));
        self.hashes.store(Arc::new(new_hashes));
        self.clear_load_note();
    }

    pub fn load_note(&self) -> Option<String> {
        self.load_note
            .read()
            .ok()
            .and_then(|note| note.clone())
    }

    pub fn set_load_note(&self, note: Option<String>) {
        if let Ok(mut slot) = self.load_note.write() {
            *slot = note;
        }
    }

    pub fn clear_load_note(&self) {
        self.set_load_note(None);
    }

    pub fn snapshot(&self) -> Arc<HashMap<String, EmbedVector>> {
        self.entries.load_full()
    }

    pub fn load_from_disk(project_root: &Path, configured_model: &str) -> Result<Self, String> {
        let db_dir = project_root.join(WM_DIR).join(STATE_DIR);
        let db_path = db_dir.join(VECTOR_DB_FILE);
        let _ = std::fs::create_dir_all(&db_dir);
        let db = vector_db::VectorDb::open(db_path, 0)
            .map_err(|e| format!("turso open error: {}", e))?;
        let db_arc = Arc::new(db);
        let (raw_entries, raw_hashes) = db_arc
            .load_all_raw()
            .map_err(|e| format!("turso load error: {}", e))?;
        let mut entries_map = HashMap::with_capacity(raw_entries.len());
        let mut hashes_map = HashMap::with_capacity(raw_entries.len());
        for (id, vec) in raw_entries {
            entries_map.insert(id.clone(), EmbedVector(vec));
            if let Some(hash_hex) = raw_hashes.get(&id) {
                let hash_bytes: [u8; 32] = hex::decode(hash_hex)
                    .ok()
                    .and_then(|v| v.try_into().ok())
                    .unwrap_or([0u8; 32]);
                hashes_map.insert(id, hash_bytes);
            }
        }
        let stored = db_arc.load_metadata().unwrap_or_default();
        let model_mismatch =
            !stored.model_name.is_empty() && stored.model_name != configured_model;
        let chunking_mismatch = !stored.chunking_version.is_empty()
            && stored.chunking_version != env!("CARGO_PKG_VERSION");
        let mismatched = model_mismatch || chunking_mismatch;

        let metadata = match mismatched {
            true => crate::EmbeddingMetadata {
                model_name: configured_model.to_string(),
                ..crate::EmbeddingMetadata::default()
            },
            false => crate::EmbeddingMetadata {
                model_name: configured_model.to_string(),
                ..stored.clone()
            },
        };
        let (entries_map, hashes_map) = match mismatched {
            true => (HashMap::new(), HashMap::new()),
            false => (entries_map, hashes_map),
        };
        let store = Self {
            entries: ArcSwap::from_pointee(entries_map),
            model_name: configured_model.to_string(),
            hashes: ArcSwap::from_pointee(hashes_map),
            db: Some(db_arc),
            embedding_metadata: std::sync::RwLock::new(metadata),
            load_note: std::sync::RwLock::new(None),
        };
        if mismatched {
            store.set_load_note(Some(format!(
                "Vector store was built with model '{}' / chunking '{}' but configured model is '{}' — vectors cleared, run `wm index rebuild`.",
                stored.model_name, stored.chunking_version, configured_model
            )));
            let _ = store.save_to_disk();
        }
        Ok(store)
    }

    pub fn save_to_disk(&self) -> Result<(), String> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| String::from("no turso database configured"))?;
        let entries_arc = self.entries.load_full();
        let hashes_arc = self.hashes.load_full();
        let raw_entries: HashMap<String, Vec<f32>> = entries_arc
            .iter()
            .map(|(k, v)| (k.clone(), v.0.clone()))
            .collect();
        let raw_hashes: HashMap<String, String> = hashes_arc
            .iter()
            .map(|(k, v)| (k.clone(), hex::encode(v)))
            .collect();
        db.store_vectors_sync(&raw_entries, &raw_hashes)
            .map_err(|e| format!("turso write error: {}", e))?;
        let mut meta = self
            .embedding_metadata
            .read()
            .map(|m| m.clone())
            .unwrap_or_default();
        meta.model_name = self.model_name.clone();
        db.store_metadata(&meta)
            .map_err(|e| format!("turso metadata write error: {}", e))?;
        Ok(())
    }

    pub fn search_turso(&self, query_vec: &[f32], limit: usize) -> Vec<(String, f32)> {
        match &self.db {
            Some(db) => db.search(query_vec, limit).unwrap_or_default(),
            None => vec![],
        }
    }

    pub fn embedding_metadata(&self) -> crate::EmbeddingMetadata {
        self.embedding_metadata
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    pub fn set_embedding_metadata(&self, meta: crate::EmbeddingMetadata) {
        if let Ok(mut m) = self.embedding_metadata.write() {
            *m = meta;
        }
    }

    pub fn remove_sections_for_page(&self, page_id: &str) {
        let prefix = format!("{}#", page_id);
        self.entries.rcu(|old| {
            let mut m = (**old).clone();
            m.retain(|id, _| !id.starts_with(&prefix));
            m
        });
        self.hashes.rcu(|old| {
            let mut m = (**old).clone();
            m.retain(|id, _| !id.starts_with(&prefix));
            m
        });
        if let Some(db) = &self.db {
            let _ = db.delete_vectors_with_prefix(&prefix);
        }
    }

    pub fn upsert_sections(
        &self,
        entries: HashMap<String, EmbedVector>,
        hashes: HashMap<String, [u8; 32]>,
    ) {
        self.entries.rcu(|old| {
            let mut m = (**old).clone();
            m.extend(entries.clone());
            m
        });
        self.hashes.rcu(|old| {
            let mut m = (**old).clone();
            m.extend(hashes.clone());
            m
        });
        if let Some(db) = &self.db {
            let raw_entries: HashMap<String, Vec<f32>> = entries
                .iter()
                .map(|(k, v)| (k.clone(), v.0.clone()))
                .collect();
            let raw_hashes: HashMap<String, String> = hashes
                .iter()
                .map(|(k, v)| (k.clone(), hex::encode(v)))
                .collect();
            let _ = db.store_vectors_raw(&raw_entries, &raw_hashes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_store(root: &Path, model: &str) {
        let store = VectorStore::new(model, root);
        let mut entries = HashMap::new();
        entries.insert("wiki:x#a".to_owned(), EmbedVector(vec![0.1, 0.2, 0.3]));
        let mut hashes = HashMap::new();
        hashes.insert("wiki:x#a".to_owned(), [7u8; 32]);
        store.replace_entries_and_hashes(entries, hashes);
        store.set_embedding_metadata(crate::EmbeddingMetadata {
            model_name: model.to_owned(),
            model_modified_at: "2026-01-01T00:00:00.000Z".to_owned(),
            chunking_version: env!("CARGO_PKG_VERSION").to_owned(),
        });
        store.save_to_disk().expect("save vectors");
    }

    #[test]
    fn mismatched_model_clears_persisted_vectors() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_store(dir.path(), "model-a");
        let loaded = VectorStore::load_from_disk(dir.path(), "model-b").expect("load");
        assert_eq!(loaded.model_name, "model-b");
        assert!(
            loaded.snapshot().is_empty(),
            "cross-model vectors must never be served"
        );
        assert!(loaded.load_note().is_some());
    }

    #[test]
    fn matching_model_restores_name_and_vectors() {
        let dir = tempfile::tempdir().expect("tempdir");
        write_store(dir.path(), "model-a");
        let loaded = VectorStore::load_from_disk(dir.path(), "model-a").expect("load");
        assert_eq!(loaded.model_name, "model-a");
        assert_eq!(loaded.snapshot().len(), 1);
        assert!(loaded.load_note().is_none());
    }
}
