pub fn yaml_scalar(value: &str) -> String {
    let rendered = serde_yaml::to_string(&serde_yaml::Value::String(value.to_string()))
        .unwrap_or_else(|_| value.to_string());
    rendered.trim_end().to_string()
}

pub fn parse_yaml_mut<F>(yaml: &str, f: F) -> String
where
    F: FnOnce(&mut serde_yaml::Mapping),
{
    let mut value: serde_yaml::Value =
        serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null);
    if value.is_null() {
        value = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
    }
    if let serde_yaml::Value::Mapping(ref mut map) = value {
        f(map);
    }
    serde_yaml::to_string(&value).unwrap_or_else(|_| yaml.to_string())
}

pub fn extract_yaml_string_value(yaml: &str, key: &str) -> String {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml).unwrap_or(serde_yaml::Value::Null);
    match value {
        serde_yaml::Value::Mapping(ref map) => {
            let k = serde_yaml::Value::String(key.to_string());
            map.get(&k)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub fn set_yaml_field(yaml: &str, key: &str, value: &str) -> String {
    parse_yaml_mut(yaml, |map| {
        map.insert(
            serde_yaml::Value::String(key.to_string()),
            serde_yaml::Value::String(value.to_string()),
        );
    })
}

pub fn ac_set_checked(yaml: &str, index: usize, checked: bool) -> String {
    parse_yaml_mut(yaml, |map| {
        if let Some(serde_yaml::Value::Sequence(ref mut items)) =
            map.get_mut(serde_yaml::Value::String("acceptance_criteria".into()))
        {
            if index > 0 && index <= items.len() {
                if let serde_yaml::Value::Mapping(ref mut ac_map) = items[index.wrapping_sub(1)] {
                    ac_map.insert(
                        serde_yaml::Value::String("checked".into()),
                        serde_yaml::Value::Bool(checked),
                    );
                }
            }
        }
    })
}

pub fn remove_yaml_block(yaml: &str, key: &str) -> String {
    parse_yaml_mut(yaml, |map| {
        map.remove(serde_yaml::Value::String(key.to_string()));
    })
}
