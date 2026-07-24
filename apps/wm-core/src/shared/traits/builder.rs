use crate::error::ToolError;

pub trait Builder<T> {
    fn build(self) -> Result<T, ToolError>;
}
