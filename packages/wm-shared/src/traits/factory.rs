use wm_error::ToolError;

/// Creates objects with construction logic (Factory pattern).
pub trait Factory<T> {
    fn create() -> Result<T, ToolError>;
}
