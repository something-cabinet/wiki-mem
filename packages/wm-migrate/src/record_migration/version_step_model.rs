#[derive(Clone, Copy, Debug)]
pub struct VersionStep {
    pub from: u32,
    pub apply: fn(&str) -> Result<String, String>,
}
