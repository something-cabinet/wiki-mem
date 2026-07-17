use schemars::JsonSchema;
use serde::Deserialize;

// ─── Action enum ────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WmPageAction {
    #[schemars(description = "List all wiki pages")]
    List {
        r#type: Option<String>,
        limit: Option<usize>,
    },
    #[schemars(description = "Get page content by ID")]
    Get { id: String },
    #[schemars(description = "Create a new wiki page")]
    Create {
        path: String,
        title: String,
        content: Option<String>,
        r#type: Option<String>,
        tags: Option<Vec<String>>,
        status: Option<String>,
    },
    #[schemars(description = "Update page frontmatter fields")]
    Update {
        id: String,
        title: Option<String>,
        content: Option<String>,
        status: Option<String>,
        tags: Option<Vec<String>>,
        r#type: Option<String>,
        relates_to: Option<Vec<serde_json::Value>>,
        notes: Option<String>,
        append_notes: Option<String>,
    },
    #[schemars(description = "Delete a page by ID")]
    Delete { id: String },
    #[schemars(description = "Add a typed edge between pages")]
    Link {
        id: String,
        target: String,
        edge_type: Option<String>,
    },
    #[schemars(description = "Remove a typed edge between pages")]
    Unlink { id: String, target: String },
}
