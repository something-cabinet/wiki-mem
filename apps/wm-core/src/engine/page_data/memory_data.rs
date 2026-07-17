use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::engine::memory::MemoryLayer;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MemoryData {
    pub layer: MemoryLayer,
    pub ttl_days: Option<u32>,
    pub last_verified: Option<String>,
    pub merged_into: Option<String>,
    pub rejected_reason: Option<String>,
    pub metadata: HashMap<String, String>,
}
