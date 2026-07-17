/// Constructs a value step by step (Builder pattern).
/// Implement on types whose purpose is building something incrementally.
/// `build()` consumes self — once built, you don't re-build.
pub trait Builder<T> {
    fn build(self) -> Result<T, wm_error::ToolError>;
}

/// Parses input into structured output (Parser pattern).
/// Implement on types that convert raw strings into structured data.
pub trait Parser<T> {
    fn parse(input: &str) -> Result<T, wm_error::ToolError>;
}

/// Creates objects with construction logic (Factory pattern).
pub trait Factory<T> {
    fn create() -> Result<T, wm_error::ToolError>;
}

/// Role marker for Repository pattern — encapsulates data access.
/// No universal method signature (save/find/delete vary per domain).
pub trait Repository {}
