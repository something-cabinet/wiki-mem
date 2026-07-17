use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WmTemplateAction {
    List {},
    Get {
        #[schemars(description = "Template name")]
        name: String,
    },
    Create {
        #[schemars(description = "Template name")]
        name: String,
        #[schemars(description = "Template description")]
        description: String,
        #[schemars(description = "Template content with {{variable}} placeholders")]
        content: String,
    },
    Run {
        #[schemars(description = "Template name")]
        name: String,
        #[schemars(description = "Variable values keyed by variable name")]
        variables: Option<std::collections::HashMap<String, String>>,
    },
}
