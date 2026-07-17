/// Marker trait for Builder pattern — constructs a value step by step.
/// Implement on types whose purpose is building something incrementally.
pub trait Builder<T> {
    fn build(self) -> Result<T, crate::error::ToolError>;
}

/// Marker trait for Parser pattern — parses input into structured output.
pub trait Parser<T> {
    fn parse(input: &str) -> Result<T, crate::error::ToolError>;
}

/// Marker trait for Repository pattern — encapsulates data access.
/// No required methods — it's a role marker for discoverability.
pub trait Repository {}

/// Marker trait for Factory pattern — creates objects with construction logic.
pub trait Factory<T> {
    fn create() -> Result<T, crate::error::ToolError>;
}
