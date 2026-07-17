use std::path::{Path, PathBuf};

/// Get the install directory path (~/.wm/bin).
pub fn install_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".wm").join("bin")
}

/// Return whether wm-cli.exe exists in the install directory.
pub fn is_installed() -> bool {
    let exe_path = install_dir().join(exe_name());
    exe_path.exists()
}

/// Copy the running binary to ~/.wm/bin/wm-cli.exe.
/// Returns the destination path on success.
pub fn install_binary() -> Result<PathBuf, String> {
    let src = std::env::current_exe()
        .map_err(|e| format!("Cannot determine current executable path: {}", e))?;
    let dst_dir = install_dir();

    std::fs::create_dir_all(&dst_dir)
        .map_err(|e| format!("Cannot create install directory {:?}: {}", dst_dir, e))?;

    let dst = dst_dir.join(exe_name());

    // If the binary is already at the destination, no-op
    if dst.exists() && src.metadata().ok().map(|m| m.len()) == dst.metadata().ok().map(|m| m.len()) {
        return Ok(dst);
    }

    std::fs::copy(&src, &dst)
        .map_err(|e| format!("Cannot copy binary to {:?}: {}", dst, e))?;

    // Verify the copy is valid
    if !dst.exists() {
        return Err(format!("Install failed: binary not found at {:?}", dst));
    }

    Ok(dst)
}

/// Add ~/.wm/bin to the user PATH if not already present.
/// On Windows, uses REG ADD HKCU\\Environment.
/// On other platforms, appends to ~/.profile (fallback).
pub fn ensure_on_path() -> Result<(), String> {
    let dir = install_dir();
    let dir_str = dir.to_str().ok_or_else(|| "Non-UTF8 install path".to_string())?;

    #[cfg(windows)]
    {
        let output = std::process::Command::new("REG")
            .args(["QUERY", "HKCU\\Environment", "/v", "PATH"])
            .output()
            .map_err(|e| format!("REG QUERY failed: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        // REG QUERY output looks like:
        //   KEY...\Environment
        //   PATH    REG_EXPAND_SZ    %USERPROFILE%\.wm\bin;...
        let already_on_path = stdout.contains(dir_str);
        if already_on_path {
            return Ok(());
        }

        // Extract current PATH value from REG QUERY output
        let current_path = stdout
            .lines()
            .find(|l| l.trim().starts_with("PATH"))
            .and_then(|l| {
                let parts: Vec<&str> = l.splitn(4, char::is_whitespace)
                    .filter(|p| !p.is_empty())
                    .collect();
                parts.get(3).map(|s| s.to_string())
            })
            .unwrap_or_default();

        let new_path = if current_path.is_empty() {
            dir_str.to_string()
        } else {
            format!("{};{}", current_path.trim(), dir_str)
        };

        let status = std::process::Command::new("REG")
            .args([
                "ADD", "HKCU\\Environment", "/v", "PATH",
                "/t", "REG_EXPAND_SZ", "/d", &new_path, "/f",
            ])
            .status()
            .map_err(|e| format!("REG ADD failed: {}", e))?;

        if !status.success() {
            return Err("Failed to update PATH via REG".to_string());
        }

        // Notify about shell restart
        println!("  Added ~\\.wm\\bin to user PATH.");
        println!("  Restart your terminal or run: refreshenv");
    }

    #[cfg(not(windows))]
    {
        let home = std::env::var("HOME").map_err(|_| "HOME not set".to_string())?;
        let profile_path = PathBuf::from(&home).join(".profile");
        let mut content = String::new();

        if profile_path.exists() {
            content = std::fs::read_to_string(&profile_path)
                .map_err(|e| format!("Cannot read {:?}: {}", profile_path, e))?;
        }

        let export_line = format!("\nexport PATH=\"$PATH:{}\"\n", dir_str);
        if content.contains(dir_str) {
            return Ok(()); // Already on PATH
        }

        content.push_str(&export_line);
        std::fs::write(&profile_path, &content)
            .map_err(|e| format!("Cannot write {:?}: {}", profile_path, e))?;
    }

    Ok(())
}

/// Check the install status — returns (installed, on_path).
pub fn check_status() -> (bool, bool) {
    let installed = is_installed();
    let on_path = is_on_path();
    (installed, on_path)
}

fn is_on_path() -> bool {
    let dir_str = install_dir().to_string_lossy().to_string();

    #[cfg(windows)]
    {
        let output = std::process::Command::new("REG")
            .args(["QUERY", "HKCU\\Environment", "/v", "PATH"])
            .output()
            .ok();
        match output {
            Some(o) => String::from_utf8_lossy(&o.stdout).contains(&dir_str),
            None => false,
        }
    }

    #[cfg(not(windows))]
    {
        std::env::var("PATH")
            .map(|p| p.split(':').any(|s| s == dir_str))
            .unwrap_or(false)
    }
}

fn exe_name() -> String {
    if cfg!(windows) {
        "wm-cli.exe".to_string()
    } else {
        "wm-cli".to_string()
    }
}
