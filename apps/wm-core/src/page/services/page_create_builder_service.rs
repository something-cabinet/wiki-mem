use std::sync::Arc;

use crate::engine::{EngineState, PageType};
use crate::error::{ToolError, ToolResult};
use crate::page::helpers::{
    build_frontmatter, set_yaml_value_field, try_render_yaml_value, FrontmatterValue,
};

const EXTRA_FRONTMATTER_LABEL: &str = "extra_frontmatter";
const RECORD_BODY_RENDER_FAILURE: &str = "record body render failed";

pub struct PageCreateRequest {
    pub path: String,
    pub page_type: PageType,
    pub fields: Vec<(&'static str, FrontmatterValue)>,
    pub content: String,
    pub extra_frontmatter: Option<serde_json::Map<String, serde_json::Value>>,
}

pub struct BuiltPage {
    pub id: String,
    pub body: String,
}

pub fn create_page_through_builder(
    engine: &Arc<EngineState>,
    request: PageCreateRequest,
) -> ToolResult<BuiltPage> {
    let body = build_create_body(&request.page_type, &request.content)?;
    let frontmatter = apply_extra_frontmatter(
        &build_frontmatter(&request.fields),
        request.extra_frontmatter.as_ref(),
    )?;
    let id = crate::page::create_page(engine, &request.path, &frontmatter, &body)?;
    Ok(BuiltPage { id, body })
}

pub fn build_create_body(page_type: &PageType, content: &str) -> ToolResult<String> {
    if !wm_engine::is_record_bearing(page_type) {
        return Ok(content.to_owned());
    }
    if wm_engine::parse_record(page_type, content).is_ok() {
        return Ok(content.to_owned());
    }
    crate::page::build_record_body(page_type, content)
        .map_err(|error| ToolError::internal(format!("{RECORD_BODY_RENDER_FAILURE}: {error}")))
}

pub fn apply_extra_frontmatter(
    frontmatter: &str,
    extra: Option<&serde_json::Map<String, serde_json::Value>>,
) -> ToolResult<String> {
    let Some(extra) = extra else {
        return Ok(frontmatter.to_owned());
    };
    let mut result = frontmatter.to_owned();
    for (key, value) in extra {
        try_render_yaml_value(value).map_err(|error| {
            ToolError::invalid_params(format!(
                "{EXTRA_FRONTMATTER_LABEL} key '{key}' cannot be rendered: {error}"
            ))
        })?;
        result = set_yaml_value_field(&result, key, value);
    }
    Ok(result)
}
