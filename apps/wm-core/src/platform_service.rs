use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// Deep-merge `new_cfg` into existing file at `path`.
/// Only the keys present in `new_cfg` are updated — all other keys
/// in the existing file are preserved at every nesting level.
///
/// If the file does not exist, just write `new_cfg` as-is.
pub fn write_merged_json(path: &Path, new_cfg: Value) -> Result<(), anyhow::Error> {
    let final_cfg = if path.exists() {
        match std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        {
            Some(mut existing) => {
                deep_merge(&mut existing, &new_cfg);
                existing
            }
            None => new_cfg,
        }
    } else {
        new_cfg
    };

    std::fs::write(path, serde_json::to_string_pretty(&final_cfg)?)?;
    Ok(())
}

fn deep_merge(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_obj), Value::Object(source_obj)) => {
            for (key, source_val) in source_obj {
                match target_obj.get_mut(key) {
                    Some(target_val) => {
                        deep_merge(target_val, source_val);
                    }
                    None => {
                        target_obj.insert(key.clone(), source_val.clone());
                    }
                }
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

/// Write a TOML config file for Codex with the format:
///
/// ```toml
/// [mcp_servers.wm]
/// command = "<bin_path>"
/// args = ["mcp"]
/// ```
pub fn write_toml_config(path: &Path, bin_path: &str) -> Result<(), anyhow::Error> {
    #[derive(Serialize)]
    struct McpServer {
        command: String,
        args: Vec<String>,
    }

    #[derive(Serialize)]
    struct McpServers {
        wm: McpServer,
    }

    #[derive(Serialize)]
    struct Config {
        #[serde(rename = "mcp_servers")]
        mcp_servers: McpServers,
    }

    let config = Config {
        mcp_servers: McpServers {
            wm: McpServer {
                command: bin_path.to_string(),
                args: vec!["mcp".to_string()],
            },
        },
    };

    let toml_string = toml::to_string(&config)?;
    std::fs::write(path, toml_string)?;
    Ok(())
}
