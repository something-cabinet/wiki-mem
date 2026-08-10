use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub trait PageRepo: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error>;
    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error>;
    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error>;
    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error>;
    fn exists(&self, path: &Path) -> bool;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error>;
    fn remove_dir(&self, path: &Path) -> Result<(), std::io::Error>;
}

pub struct FsPageRepo;

impl PageRepo for FsPageRepo {
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }
    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
        // Atomic write: write to a temp file in the same directory, then rename.
        // Concurrent graph rebuilds scan the wiki dir and can read a partially
        // written file if we wrote in place — a rename makes the file visible
        // atomically so readers never see partial content.
        let parent = path.parent().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no parent")
        })?;
        std::fs::create_dir_all(parent)?;
        let tmp = parent.join(format!(
            ".{}.tmp-{}",
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("page"),
            std::process::id()
        ));
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)
    }
    fn create_dir_all(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(path)
    }
    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::remove_file(path)
    }
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let entries = std::fs::read_dir(path)?;
        entries.map(|e| e.map(|e| e.path())).collect()
    }
    fn remove_dir(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::remove_dir(path)
    }
}

pub struct InMemoryPageRepo {
    files: Mutex<std::collections::HashMap<PathBuf, Vec<u8>>>,
}

impl Default for InMemoryPageRepo {
    fn default() -> Self {
        Self {
            files: Mutex::new(std::collections::HashMap::new()),
        }
    }
}

impl InMemoryPageRepo {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PageRepo for InMemoryPageRepo {
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
        let files = self
            .files
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        files
            .get(path)
            .map(|v| String::from_utf8_lossy(v).to_string())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"))
    }
    fn write(&self, path: &Path, content: &[u8]) -> Result<(), std::io::Error> {
        let mut files = self
            .files
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        files.insert(path.to_path_buf(), content.to_vec());
        Ok(())
    }
    fn create_dir_all(&self, _path: &Path) -> Result<(), std::io::Error> {
        Ok(())
    }
    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut files = self
            .files
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        files.remove(path);
        Ok(())
    }
    fn exists(&self, path: &Path) -> bool {
        self.files
            .lock()
            .map(|f| f.contains_key(path))
            .unwrap_or(false)
    }
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let files = self
            .files
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let prefix = path.to_path_buf();
        Ok(files
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .map(|k| k.to_path_buf())
            .collect())
    }
    fn remove_dir(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut files = self
            .files
            .lock()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        files.retain(|k, _| !k.starts_with(path));
        Ok(())
    }
}
