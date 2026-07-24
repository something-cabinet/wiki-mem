use crate::error::ToolError;

pub trait Parser<T> {
    fn parse(input: &str) -> Result<T, ToolError>;
}
