pub mod block;
pub mod template_ref;
pub mod variable;
pub(crate) mod case_helpers;

pub use block::extract_block;
pub use template_ref::parse_template_ref;
pub use variable::{resolve_variable, resolve_condition, is_truthy};
