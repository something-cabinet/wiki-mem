use serde::{Deserialize, Serialize};
pub use wm_engine::models::edge_type_model::EdgeProvenance;

/// A typed cross-file code edge (spec item 2, FR-2.1/FR-2.2).
///
/// Raw facts are extracted per file (deterministic, local — NFR-2.1) and
/// stored in the code index (`code_edges` table). Targets are resolved
/// against the global symbol index at query time:
///
/// - `imports`:  source file → target file. `target_file` carries the
///   path-math candidate from extraction; the resolver confirms it exists in
///   the index (drops external/unresolvable imports).
/// - `calls`:    source symbol → target symbol (`target_symbol` = callee,
///   `target_file` filled by resolution).
/// - `inherits`: source symbol → target symbol (`target_symbol` = base
///   class/trait/interface, `target_file` filled by resolution).
///
/// `line` is the 1-based line in `source_file` where the reference appears
/// (FR-2.3 source locations).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodeEdge {
    /// One of `calls`, `imports`, `inherits`.
    pub edge_type: String,
    /// File (relative to project root) containing the reference.
    pub source_file: String,
    /// Enclosing symbol for calls/inherits (the caller / implementing type).
    pub source_symbol: Option<String>,
    /// Resolved target file. Empty until resolution for calls/inherits.
    pub target_file: String,
    /// Callee / base name for calls/inherits; raw import path for imports.
    pub target_symbol: Option<String>,
    /// Receiver expression extracted from the AST (FR-2.2): the text that
    /// qualifies the callee so resolution is not handed a bare name.
    /// - `Some("self")` for `self.method()`
    /// - `Some("Foo")` for `Foo::assoc()` or `Foo::new()`
    /// - `Some("x")` for `x.method()` (binding; resolution infers the type)
    /// - `None` for bare `fn()` calls (no receiver)
    pub receiver: Option<String>,
    /// 1-based line of the reference in `source_file`.
    pub line: usize,
    /// `explicit` = direct AST reference; `derived` = via re-export/indirection;
    /// `ambiguous` = multi-candidate symbol resolution (FR-2.2, reuses P1 enum).
    pub provenance: EdgeProvenance,
}

impl CodeEdge {
    pub fn is_break_sensitive(&self) -> bool {
        matches!(self.edge_type.as_str(), "calls" | "imports" | "inherits")
    }
}
