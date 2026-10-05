use std::sync::{Arc, Mutex, OnceLock};

use wm_engine::DecisionError;

use super::gliner_backend::GlinerBackend;

pub struct BackendCache<B> {
    slot: Mutex<Option<(String, Arc<B>)>>,
}

impl<B> BackendCache<B> {
    pub fn new() -> Self {
        Self {
            slot: Mutex::new(None),
        }
    }

    pub fn get_or_load(
        &self,
        key: &str,
        load: impl FnOnce() -> Result<B, DecisionError>,
    ) -> Result<Arc<B>, DecisionError> {
        let mut slot = self.slot.lock().map_err(|_| DecisionError::Backend {
            detail: "backend cache lock poisoned".to_owned(),
        })?;
        if let Some((cached_key, backend)) = slot.as_ref() {
            if cached_key == key {
                return Ok(Arc::clone(backend));
            }
        }
        let backend = Arc::new(load()?);
        *slot = Some((key.to_owned(), Arc::clone(&backend)));
        Ok(backend)
    }
}

impl<B> Default for BackendCache<B> {
    fn default() -> Self {
        Self::new()
    }
}

pub fn backend_cache() -> &'static BackendCache<GlinerBackend> {
    static CACHE: OnceLock<BackendCache<GlinerBackend>> = OnceLock::new();
    CACHE.get_or_init(BackendCache::new)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn caches_by_key_and_reloads_when_the_key_changes() {
        let cache: BackendCache<u32> = BackendCache::new();
        let loads = AtomicUsize::new(0);
        let load = |value: u32| {
            loads.fetch_add(1, Ordering::SeqCst);
            Ok(value)
        };

        let first = cache.get_or_load("a", || load(1)).expect("load");
        let again = cache.get_or_load("a", || load(2)).expect("cached");
        assert_eq!(*first, 1);
        assert_eq!(*again, 1);
        assert_eq!(loads.load(Ordering::SeqCst), 1);

        let other = cache.get_or_load("b", || load(3)).expect("reload");
        assert_eq!(*other, 3);
        assert_eq!(loads.load(Ordering::SeqCst), 2);
    }
}
