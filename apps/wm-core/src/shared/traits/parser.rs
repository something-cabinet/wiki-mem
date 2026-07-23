use crate::error::ToolError;

/// Parses input into structured output (Parser pattern).
pub trait Parser<T> {
    fn parse(input: &str) -> Result<T, ToolError>;
}
