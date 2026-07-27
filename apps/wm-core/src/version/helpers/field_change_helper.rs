use crate::version::models::field_change_model;

pub fn compute_field_changes(
    old: &serde_json::Value,
    new: &serde_json::Value,
) -> Vec<field_change_model::FieldChange> {
    let old_map = old.as_object();
    let new_map = new.as_object();
    let mut changes = Vec::new();

    let mut fields: Vec<&str> = Vec::new();
    if let Some(m) = old_map {
        for key in m.keys() {
            if !fields.contains(&key.as_str()) {
                fields.push(key);
            }
        }
    }
    if let Some(m) = new_map {
        for key in m.keys() {
            if !fields.contains(&key.as_str()) {
                fields.push(key);
            }
        }
    }

    for field in fields {
        let old_val = old_map.and_then(|m| m.get(field));
        let new_val = new_map.and_then(|m| m.get(field));
        if old_val != new_val {
            changes.push(field_change_model::FieldChange {
                field: field.to_string(),
                old_value: old_val.cloned(),
                new_value: new_val.cloned(),
            });
        }
    }
    changes
}
