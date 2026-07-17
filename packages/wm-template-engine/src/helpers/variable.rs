use serde_json::Value;

/// Resolve a variable path (dot-separated) from a JSON map.
pub fn resolve_variable(name: &str, variables: &serde_json::Map<String, Value>) -> Value {
    let parts: Vec<&str> = name.split('.').collect();
    let mut current: Option<&Value> = None;

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            current = variables.get(*part);
        } else {
            match current {
                Some(Value::Object(map)) => current = map.get(*part),
                _ => return Value::Null,
            }
        }
    }

    current.cloned().unwrap_or(Value::Null)
}

/// Resolve a condition variable: handles `true`, `false`, existing variables,
/// and negated expressions like `!var`.
pub fn resolve_condition(expr: &str, variables: &serde_json::Map<String, Value>) -> Value {
    match expr.trim() {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" | "nil" | "undefined" => Value::Null,
        other => {
            if let Ok(n) = other.parse::<f64>() {
                Value::Number(serde_json::Number::from_f64(n).unwrap_or(serde_json::Number::from(0)))
            } else {
                resolve_variable(other, variables)
            }
        }
    }
}

/// Determine truthiness of a JSON value for `#if` and `#unless` blocks.
pub fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_variable() {
        let mut vars = serde_json::Map::new();
        vars.insert("name".into(), json!("World"));
        assert_eq!(resolve_variable("name", &vars), json!("World"));
    }

    #[test]
    fn test_dot_notation() {
        let mut vars = serde_json::Map::new();
        vars.insert("user".into(), json!({"name": "Alice"}));
        assert_eq!(resolve_variable("user.name", &vars), json!("Alice"));
    }

    #[test]
    fn test_unknown_variable() {
        let vars = serde_json::Map::new();
        assert_eq!(resolve_variable("unknown", &vars), Value::Null);
    }

    #[test]
    fn test_resolve_condition_true() {
        let vars = serde_json::Map::new();
        assert_eq!(resolve_condition("true", &vars), Value::Bool(true));
    }

    #[test]
    fn test_resolve_condition_false() {
        let vars = serde_json::Map::new();
        assert_eq!(resolve_condition("false", &vars), Value::Bool(false));
    }

    #[test]
    fn test_resolve_condition_variable() {
        let mut vars = serde_json::Map::new();
        vars.insert("show".into(), json!(true));
        assert_eq!(resolve_condition("show", &vars), Value::Bool(true));
    }

    #[test]
    fn test_is_truthy_values() {
        assert!(!is_truthy(&Value::Null));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_truthy(&Value::Number(1.into())));
        assert!(!is_truthy(&Value::Number(0.into())));
        assert!(is_truthy(&Value::String("hello".into())));
        assert!(!is_truthy(&Value::String("".into())));
        assert!(is_truthy(&Value::Array(vec![json!(1)])));
        assert!(!is_truthy(&Value::Array(vec![])));
    }
}
