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
    /// Persisted embedding metadata (model mtime + chunking version) used by
    /// the incremental-rebuild version-tracking triggers (#89/#74).
    pub embedding_metadata: std::sync::RwLock<crate::EmbeddingMetadata>,
}

impl VectorStore {
    pub fn new(model_name: &str, project_root: &Path) -> Self {
        let db_dir = project_root.join(WM_DIR).join(STATE_DIR);
        let db_path = db_dir.join(VECTOR_DB_FILE);
        let _ = std::fs::create_dir_all(&db_dir);
        let db = vector_db::VectorDb::open(db_path, 0).ok().map(Arc::new);
        Self {
            entries: ArcSwap::from_pointee(HashMap::new()),
            model_name: model_name.to_string(),
            hashes: ArcSwap::from_pointee(HashMap::new()),
            db,
            embedding_metadata: std::sync::RwLock::new(crate::EmbeddingMetadata::default()),
        }
    }

    pub fn replace_entries_and_hashes(
        &self,
        new_entries: HashMap<String, EmbedVector>,
        new_hashes: HashMap<String, [u8; 32]>,
    ) {
        self.entries.store(Arc::new(new_entries));
        self.hashes.store(Arc::new(new_hashes));
    }

    pub fn snapshot(&self) -> Arc<HashMap<String, EmbedVector>> {
        self.entries.load_full()
    }

    /// Load the vector store from disk (turso database).
    ///
    pub fn load_from_disk(project_root: &Path) -> Result<Self, String> {
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
        let meta = db_arc.load_metadata().unwrap_or_default();
        let store = Self {
            entries: ArcSwap::from_pointee(entries_map),
            model_name: String::new(),
            hashes: ArcSwap::from_pointee(hashes_map),
            db: Some(db_arc),
            embedding_metadata: std::sync::RwLock::new(meta),
        };
        Ok(store)
    }

    /// Save the current in-memory entries and hashes to disk, reconciling the
    /// persisted store against the in-memory map (orphan vectors for deleted
    /// pages are removed) and persisting the embedding metadata.
    ///
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
        let meta = self
            .embedding_metadata
            .read()
            .map(|m| m.clone())
            .unwrap_or_default();
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

    /// Current persisted embedding metadata.
    ///
    pub fn embedding_metadata(&self) -> crate::EmbeddingMetadata {
        self.embedding_metadata
            .read()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Update the in-memory embedding metadata (persisted on next save).
    ///
    pub fn set_embedding_metadata(&self, meta: crate::EmbeddingMetadata) {
        if let Ok(mut m) = self.embedding_metadata.write() {
            *m = meta;
        }
    }

    /// Remove every section vector belonging to a page (both in-memory and
    /// turso). Used by the incremental page-delete/update path so stale
    /// vectors never return in search.
    ///
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

    /// Upsert a set of section vectors (both in-memory and turso). Upsert-only
    /// — unrelated existing vectors are left untouched.
    ///
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
