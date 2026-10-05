use serde::Deserialize;

pub const SHA256_HEX_LEN: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct ModelFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

impl ModelFile {
    pub fn has_placeholder_hash(&self) -> bool {
        self.sha256.chars().all(|character| character == '0')
    }

    pub fn has_valid_hash(&self) -> bool {
        self.sha256.len() == SHA256_HEX_LEN
            && self
                .sha256
                .chars()
                .all(|character| character.is_ascii_hexdigit())
    }
}
