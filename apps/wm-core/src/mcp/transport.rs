
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use rmcp::model::Tool;

use crate::error::ToolError;


pub type ToolHandler = Arc<dyn Fn(Value) -> Result<Value, ToolError> + Send + Sync>;

pub type AsyncToolHandler =
    Arc<dyn Fn(Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send>> + Send + Sync>;

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
            if !obj.contains_key("type") {
                obj.insert("type".into(), serde_json::json!("object"));
            }
            obj.remove("title");
            obj.remove("description");
        }
        if let Ok(mut guard) = cache.write() {
            guard.insert(std::any::TypeId::of::<T>(), value.clone());
        }
        value
    })
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

    pub fn register_with_desc(
        &mut self,
        name: &str,
        description: &str,
        handler: ToolHandler,
    ) {
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
        self.async_handlers.contains_key(name)
            || self.handlers.iter().any(|(n, _)| n == name)
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
                let duration_ms = start.elapsed().as_millis() as i64;
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
                        "unknown panic".to_string()
                    };
                    Err(ToolError::internal(msg))
                }
            };
        } else {
            return Err(ToolError::invalid_action(&[method]));
        }

        let duration_ms = start.elapsed().as_millis() as i64;
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


