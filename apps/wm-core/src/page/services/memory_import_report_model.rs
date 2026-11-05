use super::memory_import_failure_model::MemoryImportFailure;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MemoryImportReport {
    pub imported: usize,
    pub failed: Vec<MemoryImportFailure>,
}
