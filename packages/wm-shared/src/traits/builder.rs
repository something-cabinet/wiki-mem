use wm_error::ToolError;

/// Constructs a value step by step (Builder pattern).
pub trait Builder<T> {
    fn build(self) -> Result<T, ToolError>;
}
