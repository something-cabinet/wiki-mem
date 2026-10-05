
#[derive(Clone, Debug)]
pub struct EngineState;

impl EngineState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self
    }
}
