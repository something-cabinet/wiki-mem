use serde_json::Value;

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
