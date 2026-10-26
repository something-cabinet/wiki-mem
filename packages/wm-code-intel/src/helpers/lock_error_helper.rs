use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

const LOCK_CONTENTION_MARKERS: [&str; 2] = ["Locking error", "locked by another process"];
const LOCK_REMEDY: &str = "close it or retry after it exits";
const WAL_SUFFIX: &str = "-wal";

pub fn is_lock_contention(error: &str) -> bool {
    LOCK_CONTENTION_MARKERS
        .iter()
        .any(|marker| error.contains(marker))
}

pub fn describe_lock_error(path: &Path, error: String) -> String {
    if !is_lock_contention(&error) {
        return error;
    }
    match lock_holder(path) {
        Some(holder) => with_holder(error, &holder),
        None => error,
    }
}

fn with_holder(error: String, holder: &str) -> String {
    format!("{} (held by {}; {})", error, holder, LOCK_REMEDY)
}

fn wal_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(WAL_SUFFIX);
    PathBuf::from(name)
}

fn lock_holder(path: &Path) -> Option<String> {
    let pid = lsof_pid(path).or_else(|| lsof_pid(&wal_path(path)))?;
    let name = process_name(pid).unwrap_or_else(|| "unknown process".to_string());
    Some(format!("PID {} {}", pid, name))
}

#[cfg(unix)]
fn lsof_pid(path: &Path) -> Option<u32> {
    let output = Command::new("lsof").arg("-t").arg(path).output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()?
        .trim()
        .parse()
        .ok()
}

#[cfg(not(unix))]
fn lsof_pid(_path: &Path) -> Option<u32> {
    None
}

#[cfg(unix)]
fn process_name(pid: u32) -> Option<String> {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()?;
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() {
        return None;
    }
    Some(name)
}

#[cfg(not(unix))]
fn process_name(_pid: u32) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_lock_errors_are_returned_unchanged() {
        let error = "no such table: symbols".to_string();
        assert_eq!(
            describe_lock_error(Path::new("/tmp/code.db"), error.clone()),
            error
        );
    }

    #[test]
    fn lock_contention_markers_match_turso_messages() {
        assert!(is_lock_contention(
            "Locking error: Failed locking file 'code.db-wal'. File is locked by another process"
        ));
        assert!(!is_lock_contention("database disk image is malformed"));
    }

    #[test]
    fn lock_errors_name_the_holder_and_remedy() {
        let described = with_holder(
            "Locking error: Failed locking file 'code.db-wal'".to_string(),
            "PID 42 wm-cli",
        );
        assert!(described.contains("PID 42 wm-cli"));
        assert!(described.contains(LOCK_REMEDY));
    }

    #[cfg(unix)]
    #[test]
    fn open_file_holder_is_discovered() {
        if Command::new("lsof").arg("-v").output().is_err() {
            return;
        }
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("code.db");
        std::fs::write(&path, b"x").expect("write db file");
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!("exec 3<{}; sleep 10", path.display()))
            .spawn()
            .expect("spawn file holder");

        let holder = wait_for_holder(&path);
        let _ = child.kill();
        let _ = child.wait();

        assert!(
            holder.is_some(),
            "lsof should name the process holding the open database file"
        );
    }

    #[cfg(unix)]
    fn wait_for_holder(path: &Path) -> Option<String> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if let Some(holder) = lock_holder(path) {
                return Some(holder);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        None
    }
}
