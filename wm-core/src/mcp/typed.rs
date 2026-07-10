#![warn(missing_docs)]

//! Typed, compile-time-safe MCP tool registration.
//!
//! Provides permission level marker types (`ReadOp`, `WriteOp`, `AdminOp`)
//! and `ToolRegistry` extension methods (`register_read`, `register_write`,
//! `register_admin`) that auto-derive JSON schemas from typed input structs
//! via rmcp's bundler `schemars`, eliminating manual `json!()` schema
//! declarations and the entire class of schema/handler parameter name drift bugs.
//!
//! Permission levels are enforced at the **method name** level:
//! - `register_read`  → read-only tools (search, list, get)
//! - `register_write` → state-changing tools (create, update, start/stop)
//! - `register_admin` → destructive tools (delete, remove)
//!
//! ## Migration
//! The old `register_with_schema` / `register_with_desc` / `register` APIs
//! remain available for the dynamic skills system and any code not yet migrated.

use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ToolError;

// ─── Permission level marker types ──────────────────────────────
// Sealed trait prevents downstream code from adding custom permission levels.

mod sealed {
    pub trait Sealed {}
}

/// Marker trait for MCP tool permission levels.
///
/// Implemented by [`ReadOp`], [`WriteOp`], and [`AdminOp`] to distinguish
/// read-only, state-changing, and destructive tool categories at the type level.
/// The actual enforcement is done at the **method name** level via
/// [`TypedRegister::register_read`], [`TypedRegister::register_write`],
/// and [`TypedRegister::register_admin`].
///
/// This trait is **sealed** — only `ReadOp`, `WriteOp`, and `AdminOp` can
/// implement it. External code cannot add new permission levels.
pub trait PermissionKind: sealed::Sealed + Send + Sync + 'static {}

/// Read-only permission level marker.
///
/// Tools registered with this marker (via [`TypedRegister::register_read`])
/// can search, list, and retrieve data but MUST NOT modify any state.
///
/// # Examples
/// - `wm_page.get` — retrieve page content
/// - `wm_search.query` — search wiki and memory
/// - `wm_page.list` — list all pages
pub struct ReadOp;
impl sealed::Sealed for ReadOp {}
impl PermissionKind for ReadOp {}

/// State-changing permission level marker.
///
/// Tools registered with this marker (via [`TypedRegister::register_write`])
/// create, update, start, or stop resources. They should NOT perform
/// destructive operations — use [`AdminOp`] for those.
///
/// # Examples
/// - `wm_page.create` — create a new wiki page
/// - `wm_page.update` — update page frontmatter or content
/// - `wm_page.link` — add a typed edge between pages
pub struct WriteOp;
impl sealed::Sealed for WriteOp {}
impl PermissionKind for WriteOp {}

/// Destructive permission level marker.
///
/// Tools registered with this marker (via [`TypedRegister::register_admin`])
/// perform irreversible or destructive operations such as deletion or removal.
///
/// # Examples
/// - `wm_page.delete` — delete a page and its file
pub struct AdminOp;
impl sealed::Sealed for AdminOp {}
impl PermissionKind for AdminOp {}

// ─── Typed registration extension methods on ToolRegistry ──────
// These are implemented in transport.rs alongside the ToolRegistry type.

/// Trait providing compile-time-safe typed MCP tool registration.
///
/// Implemented on [`ToolRegistry`](crate::mcp::transport::ToolRegistry) in the
/// `transport` module. The three methods distinguish permission levels by name:
///
/// | Method | Permission | Use case |
/// |--------|-----------|----------|
/// | [`register_read`](TypedRegister::register_read) | Read-only | Search, list, get |
/// | [`register_write`](TypedRegister::register_write) | State-changing | Create, update, start/stop |
/// | [`register_admin`](TypedRegister::register_admin) | Destructive | Delete, remove |
///
/// Each method auto-derives the JSON input schema from the generic type `I`
/// via `schemars`, eliminating manual `json!()` schema declarations and the
/// entire class of schema/handler parameter name drift bugs.
pub trait TypedRegister {
    /// Register a read-only MCP tool with auto-derived JSON schema.
    ///
    /// Read-only tools can search, list, and retrieve data but MUST NOT modify
    /// any state. This is enforced at the method name level.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register_read::<WmPageGetInput, WmPageGetOutput>(
    ///     "wm_page.get",
    ///     "Get page content by ID",
    ///     move |input| {
    ///         let content = page::get_page(&engine, &input.id)?;
    ///         Ok(WmPageGetOutput {
    ///             id: input.id,
    ///             content: content.raw,
    ///             sections: vec![],
    ///         })
    ///     },
    /// );
    /// ```
    ///
    /// # Type Parameters
    ///
    /// - `I`: Input type — must implement [`DeserializeOwned`] and [`JsonSchema`].
    ///   The JSON schema for `I` is auto-derived via `schemars`.
    /// - `O`: Output type — must implement [`Serialize`]. The handler result is
    ///   serialized to JSON and sent back to the MCP client.
    ///
    /// # Permission
    ///
    /// Read-only tools cannot modify state. Use [`register_write`](TypedRegister::register_write)
    /// for state-changing tools and [`register_admin`](TypedRegister::register_admin)
    /// for destructive operations.
    fn register_read<I, O>(
        &mut self,
        name: &'static str,
        description: &'static str,
        handler: impl Fn(I) -> Result<O, ToolError> + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static;

    /// Register a state-changing MCP tool with auto-derived JSON schema.
    ///
    /// Write tools can create, update, start, stop, or otherwise modify state.
    /// They should NOT perform destructive actions (deletion, removal) —
    /// use [`register_admin`](TypedRegister::register_admin) for those.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register_write::<WmPageCreateInput, WmPageCreateOutput>(
    ///     "wm_page.create",
    ///     "Create a new wiki page",
    ///     move |input| {
    ///         let id = page::create_page(&engine, &input.path, &frontmatter, &content)?;
    ///         Ok(WmPageCreateOutput {
    ///             id,
    ///             path: input.path,
    ///             r#type: "concept".into(),
    ///         })
    ///     },
    /// );
    /// ```
    ///
    /// # Type Parameters
    ///
    /// - `I`: Input type — must implement [`DeserializeOwned`] and [`JsonSchema`].
    /// - `O`: Output type — must implement [`Serialize`].
    ///
    /// # Permission
    ///
    /// State-changing tools may modify data but must not perform destructive
    /// operations. Use [`register_admin`](TypedRegister::register_admin) for
    /// deletion and other irreversible actions.
    fn register_write<I, O>(
        &mut self,
        name: &'static str,
        description: &'static str,
        handler: impl Fn(I) -> Result<O, ToolError> + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static;

    /// Register a destructive MCP tool with auto-derived JSON schema.
    ///
    /// Admin tools perform irreversible or destructive operations such as
    /// deletion, removal, or purging. Use this level sparingly.
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register_admin::<WmPageDeleteInput, WmPageDeleteOutput>(
    ///     "wm_page.delete",
    ///     "Delete a page and its file",
    ///     move |input| {
    ///         // ... delete logic ...
    ///         Ok(WmPageDeleteOutput {
    ///             id: input.id,
    ///             status: "deleted".into(),
    ///         })
    ///     },
    /// );
    /// ```
    ///
    /// # Type Parameters
    ///
    /// - `I`: Input type — must implement [`DeserializeOwned`] and [`JsonSchema`].
    /// - `O`: Output type — must implement [`Serialize`].
    ///
    /// # Permission
    ///
    /// Destructive tools can irreversibly modify or delete data. Use
    /// [`register_read`](TypedRegister::register_read) for read-only tools and
    /// [`register_write`](TypedRegister::register_write) for state-changing tools.
    fn register_admin<I, O>(
        &mut self,
        name: &'static str,
        description: &'static str,
        handler: impl Fn(I) -> Result<O, ToolError> + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static;
}
