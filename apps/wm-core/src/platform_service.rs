use serde::Serialize;
use serde_json::Value;
use std::path::Path;

/// Read existing JSON file at `path`, deserialize it, merge with `new_cfg`
/// (new top-level keys overwrite existing, but existing keys not in `new_cfg`
/// are preserved), re-serialize and write.
///
/// If the file does not exist, just write `new_cfg` as-is.
pub fn write_merged_json(path: &Path, new_cfg: Value) -> Result<(), anyhow::Error> {
    let final_cfg = if path.exists() {
        match std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        {
            Some(mut existing) => {
                // Merge new_cfg on top at the top level
                if let Some(new_obj) = new_cfg.as_object() {
                    if let Some(existing_obj) = existing.as_object_mut() {
                        for (key, value) in new_obj {
                            existing_obj.insert(key.clone(), value.clone());
                        }
                    }
                }
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
