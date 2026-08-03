use rmcp::model::Tool;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::error::ToolError;

const ACTION_FIELD: &str = "action";
const SCHEMA_CONST: &str = "const";
const SCHEMA_DESCRIPTION: &str = "description";
const SCHEMA_ENUM: &str = "enum";
const SCHEMA_ONE_OF: &str = "oneOf";
const SCHEMA_PROPERTIES: &str = "properties";
const SCHEMA_REQUIRED: &str = "required";
const SCHEMA_TYPE: &str = "type";
const SCHEMA_TYPE_OBJECT: &str = "object";
const SCHEMA_TYPE_STRING: &str = "string";

pub type ToolHandler = Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>;

pub type AsyncToolHandler = Arc<
    dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send>> + Send + Sync,
>;

pub type AuditCallback =
    Arc<dyn Fn(&str, &str, &str, i64, Option<String>, Vec<String>) + Send + Sync>;

pub type PermissionCheck = Arc<dyn Fn(&str) -> bool + Send + Sync>;

pub struct ToolRegistry {
    handlers: Vec<(String, ToolHandler)>,
    async_handlers: HashMap<String, AsyncToolHandler>,
    descriptions: HashMap<String, String>,
    schemas: HashMap<String, Value>,
    audit: Option<AuditCallback>,
    pub check_permission: Option<PermissionCheck>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_input_schema<T: JsonSchema + 'static>() -> serde_json::Value {
    thread_local! {
        static CACHE: std::sync::RwLock<HashMap<std::any::TypeId, serde_json::Value>> = std::sync::RwLock::new(HashMap::new());
    }
    CACHE.with(|cache| {
        if let Ok(guard) = cache.read() {
            if let Some(schema) = guard.get(&std::any::TypeId::of::<T>()) {
                return schema.clone();
            }
        }
        let settings = schemars::generate::SchemaSettings::draft2020_12();
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<T>();
        let mut value = serde_json::to_value(schema).unwrap_or_default();
        if let Some(obj) = value.as_object_mut() {
            if !obj.contains_key(SCHEMA_TYPE) {
                obj.insert(SCHEMA_TYPE.into(), SCHEMA_TYPE_OBJECT.into());
            }
            obj.remove("title");
            obj.remove(SCHEMA_DESCRIPTION);
        }
        flatten_tagged_enum_schema(&mut value);
        if let Ok(mut guard) = cache.write() {
            guard.insert(std::any::TypeId::of::<T>(), value.clone());
        }
        value
    })
}

fn flatten_tagged_enum_schema(value: &mut Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };
    let Some(arms) = obj.get(SCHEMA_ONE_OF).and_then(Value::as_array) else {
        return;
    };
    let mut action_values: Vec<Value> = Vec::new();
    let mut merged: Map<String, Value> = Map::new();
    for arm in arms {
        let Some(props) = arm.get(SCHEMA_PROPERTIES).and_then(Value::as_object) else {
            continue;
        };
        for (field, prop) in props {
            if field == ACTION_FIELD {
                collect_action_value(prop, &mut action_values);
            } else {
                merge_property(field, prop, &mut merged);
            }
        }
    }
    let mut action_prop = Map::new();
    action_prop.insert(SCHEMA_TYPE.into(), SCHEMA_TYPE_STRING.into());
    if !action_values.is_empty() {
        action_prop.insert(SCHEMA_ENUM.into(), Value::Array(action_values));
    }
    merged.insert(ACTION_FIELD.into(), Value::Object(action_prop));
    obj.insert(SCHEMA_PROPERTIES.into(), Value::Object(merged));
    obj.insert(SCHEMA_REQUIRED.into(), serde_json::json!([ACTION_FIELD]));
    obj.remove(SCHEMA_ONE_OF);
}

fn collect_action_value(prop: &Value, out: &mut Vec<Value>) {
    if let Some(constant) = prop.get(SCHEMA_CONST) {
        out.push(constant.clone());
        return;
    }
    if let Some(values) = prop.get(SCHEMA_ENUM).and_then(Value::as_array) {
        out.extend(values.iter().cloned());
    }
}

fn merge_property(field: &str, prop: &Value, merged: &mut Map<String, Value>) {
    if !merged.contains_key(field) {
        merged.insert(field.to_string(), prop.clone());
        return;
    }
    if let Some(existing) = merged.get_mut(field).and_then(Value::as_object_mut) {
        if existing.get(SCHEMA_DESCRIPTION).is_none() {
            if let Some(desc) = prop.get(SCHEMA_DESCRIPTION) {
                existing.insert(SCHEMA_DESCRIPTION.to_string(), desc.clone());
            }
        }
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            async_handlers: HashMap::new(),
            descriptions: HashMap::new(),
            schemas: HashMap::new(),
            audit: None,
            check_permission: None,
        }
    }

    pub fn set_audit(&mut self, cb: AuditCallback) {
        self.audit = Some(cb);
    }

    pub fn set_permission_check(&mut self, cb: Arc<dyn Fn(&str) -> bool + Send + Sync>) {
        self.check_permission = Some(cb);
    }

    pub fn register(&mut self, name: &str, handler: ToolHandler) {
        self.handlers.push((name.to_string(), handler));
    }

    pub fn register_with_desc(&mut self, name: &str, description: &str, handler: ToolHandler) {
        self.descriptions
            .insert(name.to_string(), description.to_string());
        self.handlers.push((name.to_string(), handler));
    }

    pub fn register_with_schema(
        &mut self,
        name: &str,
        description: &str,
        schema: Value,
        handler: ToolHandler,
    ) {
        self.schemas.insert(name.to_string(), schema);
        self.descriptions
            .insert(name.to_string(), description.to_string());
        self.handlers.push((name.to_string(), handler));
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.async_handlers.contains_key(name) || self.handlers.iter().any(|(n, _)| n == name)
    }

    pub fn list_tools(&self) -> Vec<Tool> {
        let mut names: Vec<String> = self.handlers.iter().map(|(n, _)| n.clone()).collect();
        for name in self.async_handlers.keys() {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
            .into_iter()
            .map(|name| {
                let desc = self.descriptions.get(&name).cloned().unwrap_or_default();
                let mut schema = self
                    .schemas
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({"type": "object", "properties": {}}));
                if let Some(obj) = schema.as_object_mut() {
                    if !obj.contains_key("type") {
                        obj.insert("type".into(), serde_json::json!("object"));
                    }
                }
                let input_schema = schema.as_object().cloned().unwrap_or_default();
                Tool::new(name, desc, input_schema)
            })
            .collect()
    }

    pub fn dispatch(&self, method: &str, params: Value) -> Result<Value, ToolError> {
        if let Some(ref check) = self.check_permission {
            if !check(method) {
                return Err(ToolError::internal("Action not permitted"));
            }
        }

        for (name, handler) in &self.handlers {
            if name == method {
                let start = std::time::Instant::now();
                let result = handler(params);
                let duration_ms = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
                if let Some(ref audit) = self.audit {
                    if *name != "wm_help" && *name != "wm_initial" {
                        let error_msg = match &result {
                            Ok(_) => None,
                            Err(e) => Some(e.to_string()),
                        };
                        let status = if error_msg.is_some() { "error" } else { "ok" };
                        let action = name.split('.').nth(1).unwrap_or("unknown");
                        audit(name, action, status, duration_ms, error_msg, Vec::new());
                    }
                }
                return result;
            }
        }
        Err(ToolError::invalid_action(&[method]))
    }

    pub async fn dispatch_async(&self, method: &str, params: Value) -> Result<Value, ToolError> {
        if let Some(ref check) = self.check_permission {
            if !check(method) {
                return Err(ToolError::internal("Action not permitted"));
            }
        }

        let start = std::time::Instant::now();
        let result: Result<Value, ToolError>;

        if let Some(handler) = self.async_handlers.get(method) {
            result = handler(params).await;
        } else if let Some((_, handler)) = self.handlers.iter().find(|(n, _)| n == method) {
            let handler = handler.clone();
            result = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                tokio::task::block_in_place(move || handler(params))
            })) {
                Ok(r) => r,
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".into()
                    };
                    Err(ToolError::internal(msg))
                }
            };
        } else {
            return Err(ToolError::invalid_action(&[method]));
        }

        let duration_ms = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
        if let Some(ref audit) = self.audit {
            if method != "wm_help" && method != "wm_initial" {
                let error_msg = match &result {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                };
                let status = if error_msg.is_some() { "error" } else { "ok" };
                let action = method.split('.').nth(1).unwrap_or("unknown");
                audit(method, action, status, duration_ms, error_msg, Vec::new());
            }
        }

        result
    }
}

use crate::error::ToolError as TE;

impl ToolRegistry {
    pub fn register_typed<I, O>(
        &mut self,
        name: &'static str,
        description: &'static str,
        handler: impl Fn(I) -> Result<O, TE> + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static,
    {
        self.schemas
            .insert(name.to_string(), generate_input_schema::<I>());
        self.descriptions
            .insert(name.to_string(), description.to_string());
        self.handlers.push((
            name.to_string(),
            Arc::new(move |params| {
                let input: I = serde_json::from_value(params)
                    .map_err(|e| TE::serde_error("deserialize tool input", e))?;
                let output = handler(input)?;
                serde_json::to_value(output)
                    .map_err(|e| TE::serde_error("serialize tool output", e))
            }),
        ));
    }

    pub fn register_typed_async<I, O, F, Fut>(
        &mut self,
        name: &'static str,
        description: &'static str,
        handler: F,
    ) where
        I: DeserializeOwned + JsonSchema + 'static,
        O: Serialize + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<O, TE>> + Send,
    {
        self.schemas
            .insert(name.to_string(), generate_input_schema::<I>());
        self.descriptions
            .insert(name.to_string(), description.to_string());

        let handler = Arc::new(handler);

        self.async_handlers.insert(
            name.to_string(),
            Arc::new(move |params: Value| {
                let handler = handler.clone();
                Box::pin(async move {
                    let input: I = serde_json::from_value(params)
                        .map_err(|e| TE::serde_error("deserialize tool input", e))?;
                    let output = handler(input).await?;
                    serde_json::to_value(output)
                        .map_err(|e| TE::serde_error("serialize tool output", e))
                })
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::flatten_tagged_enum_schema;
    use serde_json::{json, Value};

    fn flatten(input: Value) -> Value {
        let mut value = input;
        flatten_tagged_enum_schema(&mut value);
        value
    }

    #[test]
    fn tagged_enum_arms_flatten_to_action_object() {
        let input = json!({
            "type": "object",
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "list", "type": "string" },
                        "state": { "type": ["string", "null"], "description": "filter state" }
                    },
                    "required": ["action"]
                },
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "get", "type": "string" },
                        "id": { "type": "string", "description": "entry id" }
                    },
                    "required": ["action", "id"]
                }
            ]
        });
        let output = flatten(input);
        assert!(output.get("oneOf").is_none());
        assert_eq!(output["type"], json!("object"));
        assert_eq!(output["required"], json!(["action"]));
        let props = output["properties"].as_object().unwrap();
        assert_eq!(
            props["action"],
            json!({ "type": "string", "enum": ["list", "get"] })
        );
        assert_eq!(
            props["state"],
            json!({ "type": ["string", "null"], "description": "filter state" })
        );
        assert_eq!(
            props["id"],
            json!({ "type": "string", "description": "entry id" })
        );
    }

    #[test]
    fn non_tagged_schema_is_untouched() {
        let input = json!({
            "type": "object",
            "properties": { "q": { "type": "string" } },
            "required": ["q"]
        });
        assert_eq!(flatten(input.clone()), input);
    }

    #[test]
    fn conflicting_field_types_are_first_wins() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "a", "type": "string" },
                        "value": { "type": "string", "description": "as string" }
                    },
                    "required": ["action"]
                },
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "b", "type": "string" },
                        "value": { "type": "number", "description": "as number" }
                    },
                    "required": ["action", "value"]
                }
            ]
        });
        let output = flatten(input);
        let value = &output["properties"]["value"];
        assert_eq!(value["type"], json!("string"));
        assert_eq!(value["description"], json!("as string"));
    }

    #[test]
    fn missing_description_is_filled_from_later_arm() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "a", "type": "string" },
                        "id": { "type": "string" }
                    },
                    "required": ["action"]
                },
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "b", "type": "string" },
                        "id": { "type": "string", "description": "shared id" }
                    },
                    "required": ["action", "id"]
                }
            ]
        });
        let output = flatten(input);
        assert_eq!(
            output["properties"]["id"]["description"],
            json!("shared id")
        );
    }

    #[test]
    fn existing_description_is_not_clobbered() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "a", "type": "string" },
                        "id": { "type": "string", "description": "first" }
                    },
                    "required": ["action"]
                },
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "b", "type": "string" },
                        "id": { "type": "string", "description": "second" }
                    },
                    "required": ["action", "id"]
                }
            ]
        });
        let output = flatten(input);
        assert_eq!(output["properties"]["id"]["description"], json!("first"));
    }

    #[test]
    fn action_enum_values_collected_from_enum_arm() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "enum": ["run", "stop"], "type": "string" },
                        "cmd": { "type": "string" }
                    },
                    "required": ["action"]
                }
            ]
        });
        let output = flatten(input);
        assert_eq!(
            output["properties"]["action"]["enum"],
            json!(["run", "stop"])
        );
    }

    #[test]
    fn arms_without_action_yield_type_only_action() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": { "cmd": { "type": "string" } }
                }
            ]
        });
        let output = flatten(input);
        assert_eq!(output["properties"]["action"], json!({ "type": "string" }));
        assert_eq!(output["required"], json!(["action"]));
    }

    #[test]
    fn defs_are_preserved_for_ref_resolution() {
        let input = json!({
            "$defs": { "PageStatus": { "type": "string", "enum": ["draft", "done"] } },
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "create", "type": "string" },
                        "status": {
                            "anyOf": [
                                { "$ref": "#/$defs/PageStatus" },
                                { "type": "null" }
                            ]
                        }
                    },
                    "required": ["action"]
                }
            ]
        });
        let output = flatten(input);
        assert!(output.get("$defs").is_some());
        assert_eq!(
            output["properties"]["status"]["anyOf"][0],
            json!({ "$ref": "#/$defs/PageStatus" })
        );
    }

    #[test]
    fn empty_properties_arm_is_skipped() {
        let input = json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "action": { "const": "list", "type": "string" }
                    },
                    "required": ["action"]
                },
                { "type": "object" }
            ]
        });
        let output = flatten(input);
        assert_eq!(output["properties"]["action"]["enum"], json!(["list"]));
    }
}
