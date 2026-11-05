use wm_core::page::MemoryImportFailure;

#[derive(Debug, Default)]
pub struct MemoryStep {
    pub imported: usize,
    pub pending: usize,
    pub failed: Vec<MemoryImportFailure>,
}
