//! Stub [`EngineState`] used by the wm-server HTTP daemon.
//!
//! This is a placeholder that will be wired to real engine internals later.

/// Minimal engine state for the HTTP server.
#[derive(Clone, Debug)]
pub struct EngineState;

impl EngineState {
    /// Create a new engine state.
    ///
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self
    }
}
