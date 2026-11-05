#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryImportFailure {
    pub file: String,
    pub reason: String,
}
