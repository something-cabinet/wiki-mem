use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagedManifest {
    pub files: BTreeMap<String, ManagedEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedEntry {
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedStatus {
    UpToDate,
    Record,
    Update,
    Edited,
    Unmanaged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManagedScope {
    Whole,
    Block,
}

#[derive(Debug, Clone)]
pub struct ManagedFilePlan {
    pub path: String,
    pub status: ManagedStatus,
    pub scope: ManagedScope,
    pub template: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct ManagedPlan {
    pub files: Vec<ManagedFilePlan>,
}

impl ManagedPlan {
    pub fn count(&self, status: ManagedStatus) -> usize {
        self.files
            .iter()
            .filter(|file| file.status == status)
            .count()
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ManagedApplied {
    pub updated: usize,
    pub recorded: usize,
}
