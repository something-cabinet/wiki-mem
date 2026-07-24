use schemars::JsonSchema;
use serde::Deserialize;


#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
#[schemars(description = "Wiki page CRUD operations: list, get, create, update, delete, link, unlink")]
pub enum WmPageAction {
    #[schemars(description = "List all wiki pages")]
    List {
        #[schemars(description = "Page type filter (task, spec, concept, etc.)")]
        r#type: Option<String>,
    },
    #[schemars(description = "Get page content by ID")]
    Get {
        #[schemars(description = "Page ID (e.g. wiki:specs:my-spec)")]
        id: String,
    },
    #[schemars(description = "Create a new wiki page")]
    Create {
        #[schemars(description = "Path relative to wiki root (e.g. specs/my-spec)")]
        path: String,
        #[schemars(description = "Page title")]
        title: String,
        #[schemars(description = "Page body content (markdown)")]
        content: Option<String>,
        #[schemars(description = "Page type (task, spec, concept, etc.)")]
        r#type: Option<String>,
        #[schemars(description = "Tags for categorization")]
        tags: Option<Vec<String>>,
        #[schemars(description = "Page status (draft, active, done, etc.)")]
        status: Option<String>,
    },
    #[schemars(description = "Update page frontmatter fields")]
    Update {
        #[schemars(description = "Page ID (e.g. wiki:specs:my-spec)")]
        id: String,
        #[schemars(description = "Page title")]
        title: Option<String>,
        #[schemars(description = "Page body content (markdown)")]
        content: Option<String>,
        #[schemars(description = "Page status (draft, active, done, etc.)")]
        status: Option<String>,
        #[schemars(description = "Tags for categorization")]
        tags: Option<Vec<String>>,
        #[schemars(description = "Page type (task, spec, concept, etc.)")]
        r#type: Option<String>,
        #[schemars(description = "Related page edges")]
        relates_to: Option<Vec<serde_json::Value>>,
        #[schemars(description = "Implementation notes (replaces body)")]
        notes: Option<String>,
        #[schemars(description = "Appends to implementation notes")]
        append_notes: Option<String>,
    },
    #[schemars(description = "Delete a page by ID")]
    Delete {
        #[schemars(description = "Page ID (e.g. wiki:specs:my-spec)")]
        id: String,
    },
    #[schemars(description = "Add a typed edge between pages")]
    Link {
        #[schemars(description = "Page ID (e.g. wiki:specs:my-spec)")]
        id: String,
        #[schemars(description = "Target page ID to link/unlink")]
        target: String,
        #[schemars(description = "Edge type (extends, depends_on, etc.)")]
        edge_type: Option<String>,
    },
    #[schemars(description = "Remove a typed edge between pages")]
    Unlink {
        #[schemars(description = "Page ID (e.g. wiki:specs:my-spec)")]
        id: String,
        #[schemars(description = "Target page ID to link/unlink")]
        target: String,
    },
}
