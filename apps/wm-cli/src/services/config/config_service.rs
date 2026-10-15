use std::path::{Path, PathBuf};

use crate::shared::models::ConfigAction;
use crate::shared::traits::Command;
use wm_constants::{CONFIG_FILE, WM_DIR};
use wm_core::config;

impl Command for ConfigAction {
    async fn run(self) -> Result<(), anyhow::Error> {
        run_config_action(self)
    }
}

fn run_config_action(action: ConfigAction) -> Result<(), anyhow::Error> {
    match action {
        ConfigAction::Get { key, json } => get_config_value(&key, json),
        ConfigAction::Set { key, value, json } => set_config_value(&key, &value, json),
        ConfigAction::List { json } => list_config(json),
    }
}

fn config_file_path() -> PathBuf {
    let root = config::detect_project_root().unwrap_or_else(|| PathBuf::from("."));
    root.join(WM_DIR).join(CONFIG_FILE)
}

fn config_pointer(key: &str) -> String {
    format!("/{}", key.replace('.', "/"))
}

fn load_config_value(path: &Path) -> Result<serde_json::Value, anyhow::Error> {
    if !path.exists() {
        anyhow::bail!("Config file not found: {}", path.display());
    }
    let content = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn get_config_value(key: &str, json: bool) -> Result<(), anyhow::Error> {
    let path = config_file_path();
    let config_value = load_config_value(&path)?;
    let Some(value) = config_value.pointer(&config_pointer(key)) else {
        anyhow::bail!("Config key not found: {}", key);
    };
    print_config_entry(key, value, json)?;
    Ok(())
}

fn print_config_entry(
    key: &str,
    value: &serde_json::Value,
    json: bool,
) -> Result<(), anyhow::Error> {
    if json {
        println!("{}", serde_json::to_string_pretty(value)?);
        return Ok(());
    }
    println!("{}: {}", key, value);
    Ok(())
}

fn set_config_value(key: &str, value: &str, json: bool) -> Result<(), anyhow::Error> {
    let path = config_file_path();
    let mut config_value = load_config_value(&path)?;
    let parsed = parse_config_value(value);
    let Some(target) = config_value.pointer_mut(&config_pointer(key)) else {
        anyhow::bail!("Cannot set key: {}", key);
    };
    *target = parsed.clone();
    std::fs::write(&path, serde_json::to_string_pretty(&config_value)?)?;
    print_set_result(key, &parsed, json)?;
    Ok(())
}

fn parse_config_value(value: &str) -> serde_json::Value {
    serde_json::from_str(value).unwrap_or_else(|_| serde_json::Value::String(value.to_owned()))
}

fn print_set_result(key: &str, value: &serde_json::Value, json: bool) -> Result<(), anyhow::Error> {
    if json {
        let payload = serde_json::json!({"ok": true, "key": key, "value": value});
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }
    println!("  {} = {}", key, value);
    Ok(())
}

fn list_config(json: bool) -> Result<(), anyhow::Error> {
    let path = config_file_path();
    let config_value = load_config_value(&path)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&config_value)?);
        return Ok(());
    }
    print_config_tree(&config_value, "");
    Ok(())
}

fn print_config_tree(value: &serde_json::Value, prefix: &str) {
    let serde_json::Value::Object(map) = value else {
        println!("  {}: {}", prefix, value);
        return;
    };
    for (key, child) in map {
        let path = qualify_config_path(prefix, key);
        print_config_tree(child, &path);
    }
}

fn qualify_config_path(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        return key.to_owned();
    }
    format!("{}.{}", prefix, key)
}
