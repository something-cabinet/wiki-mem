use std::path::{Component, Path, PathBuf};

use crate::error::{ToolError, ToolResult};
use crate::shared::audit_sink::{self, KIND_HIDDEN_PATH, KIND_PATH_ESCAPE};

const ERR_ESCAPES_ROOT: &str = "Access denied: path escapes the allowed root";
const ERR_HIDDEN: &str = "Access denied: dotfiles and hidden directories are not allowed";
const DOT_PREFIX: char = '.';
const CURRENT_DIR: &str = ".";

/// Build the escape rejection, naming the offending candidate path so callers
/// (e.g. the template runner) can surface which variable produced it.
fn escape_error(candidate: &Path) -> ToolError {
    ToolError::invalid_params(format!(
        "{} (rejected path: {})",
        ERR_ESCAPES_ROOT,
        audit_sink::sanitize(&candidate.to_string_lossy())
    ))
}

pub fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out: Vec<Component> = Vec::new();
    for component in path.components() {
        if matches!(component, Component::CurDir) {
            continue;
        }
        if matches!(component, Component::ParentDir)
            && matches!(out.last(), Some(Component::Normal(_)))
        {
            out.pop();
            continue;
        }
        out.push(component);
    }
    out.iter().collect()
}

pub fn is_confined(root: &Path, candidate: &Path) -> bool {
    let root_norm = normalize_lexically(root);
    normalize_lexically(&root.join(candidate)).starts_with(&root_norm)
}

fn symlink_escapes(root: &Path, resolved: &Path) -> bool {
    let Ok(real_root) = root.canonicalize() else {
        return false;
    };
    let mut probe = resolved.to_path_buf();
    loop {
        if probe.exists() {
            break;
        }
        match probe.parent() {
            Some(parent) => probe = parent.to_path_buf(),
            None => return false,
        }
    }
    let Ok(real) = probe.canonicalize() else {
        return false;
    };
    !real.starts_with(&real_root)
}

pub fn confine(root: &Path, candidate: &Path) -> ToolResult<PathBuf> {
    let root_norm = normalize_lexically(root);
    let resolved = normalize_lexically(&root.join(candidate));

    if !resolved.starts_with(&root_norm) {
        tracing::warn!(
            "Rejected path outside root: candidate={} root={}",
            candidate.display(),
            root_norm.display()
        );
        audit_sink::audit_confine_rejection(&root_norm, candidate, KIND_PATH_ESCAPE);
        return Err(escape_error(candidate));
    }

    if symlink_escapes(&root_norm, &resolved) {
        tracing::warn!(
            "Rejected symlink escaping root: candidate={}",
            candidate.display()
        );
        audit_sink::audit_confine_rejection(&root_norm, candidate, KIND_PATH_ESCAPE);
        return Err(escape_error(candidate));
    }

    Ok(resolved)
}

pub fn confine_strict(root: &Path, candidate: &Path) -> ToolResult<PathBuf> {
    let resolved = confine(root, candidate)?;
    let root_norm = normalize_lexically(root);
    let Ok(tail) = resolved.strip_prefix(&root_norm) else {
        return Err(ToolError::invalid_params(ERR_ESCAPES_ROOT));
    };

    let hidden = tail.components().any(|c| {
        c.as_os_str()
            .to_str()
            .is_some_and(|s| s.starts_with(DOT_PREFIX) && s != CURRENT_DIR)
    });

    if hidden {
        tracing::warn!(
            "Rejected hidden path: candidate={}",
            candidate.display()
        );
        audit_sink::audit_confine_rejection(&root_norm, candidate, KIND_HIDDEN_PATH);
        return Err(ToolError::invalid_params(format!(
            "{} (rejected path: {})",
            ERR_HIDDEN,
            audit_sink::sanitize(&candidate.to_string_lossy())
        )));
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = ".wm/wiki";

    #[test]
    fn rejects_leading_parent_traversal() {
        assert!(confine(Path::new(ROOT), Path::new("../../etc/passwd.md")).is_err());
    }

    #[test]
    fn rejects_repeated_parent_traversal() {
        assert!(confine(Path::new(ROOT), Path::new("../../../../../../tmp/x.md")).is_err());
    }

    #[test]
    fn rejects_mid_path_traversal() {
        assert!(confine(Path::new(ROOT), Path::new("specs/../../../etc/x.md")).is_err());
    }

    #[test]
    fn rejects_absolute_outside_root() {
        assert!(confine(Path::new(ROOT), Path::new("/etc/passwd")).is_err());
    }

    #[test]
    fn allows_nested_relative_path() {
        let out = confine(Path::new(ROOT), Path::new("specs/my-spec.md")).expect("should allow");
        assert_eq!(out, PathBuf::from(".wm/wiki/specs/my-spec.md"));
    }

    #[test]
    fn allows_nonexistent_create_path() {
        let out = confine(Path::new(ROOT), Path::new("tasks/not-created-yet.md"))
            .expect("create-paths must be allowed");
        assert!(out.starts_with(ROOT));
    }

    #[test]
    fn result_stays_relative_when_root_is_relative() {
        let out = confine(Path::new(ROOT), Path::new("a/b.md")).expect("ok");
        assert!(!out.is_absolute(), "must not absolutise: {:?}", out);
    }

    #[test]
    fn interior_traversal_that_stays_inside_is_allowed() {
        let out = confine(Path::new(ROOT), Path::new("specs/../tasks/x.md")).expect("ok");
        assert_eq!(out, PathBuf::from(".wm/wiki/tasks/x.md"));
    }

    #[test]
    fn strict_rejects_dotfiles() {
        assert!(confine_strict(Path::new("."), Path::new(".git/config")).is_err());
        assert!(confine_strict(Path::new("."), Path::new(".env")).is_err());
    }

    #[test]
    fn strict_allows_plain_paths_under_dot_root() {
        let out = confine_strict(Path::new(ROOT), Path::new("specs/x.md")).expect("ok");
        assert_eq!(out, PathBuf::from(".wm/wiki/specs/x.md"));
    }

    #[test]
    fn normalize_collapses_current_and_parent() {
        assert_eq!(
            normalize_lexically(Path::new("a/./b/../c")),
            PathBuf::from("a/c")
        );
    }

    #[test]
    fn normalize_keeps_escaping_parents() {
        assert_eq!(
            normalize_lexically(Path::new("a/../../b")),
            PathBuf::from("../b")
        );
    }

    #[test]
    fn is_confined_matches_confine() {
        assert!(is_confined(Path::new(ROOT), Path::new("ok/x.md")));
        assert!(!is_confined(Path::new(ROOT), Path::new("../escape.md")));
    }

    #[test]
    fn empty_candidate_resolves_to_root() {
        let out = confine(Path::new(ROOT), Path::new("")).expect("empty is the root itself");
        assert_eq!(out, PathBuf::from(".wm/wiki"));
    }

    #[test]
    fn dot_only_candidate_resolves_to_root() {
        let out = confine(Path::new(ROOT), Path::new(".")).expect("'.' is the root itself");
        assert_eq!(out, PathBuf::from(".wm/wiki"));
    }

    #[test]
    fn windows_backslash_separators_cannot_escape() {
        let out = confine(Path::new(ROOT), Path::new("..\\..\\etc"))
            .expect("backslash traversal stays a single literal component");
        assert!(
            out.starts_with(ROOT),
            "must remain under the root, got: {:?}",
            out
        );
    }

    #[test]
    fn absolute_inside_root_is_allowed() {
        let tmp = std::env::temp_dir().join(format!("wm-confine-abs-{}", std::process::id()));
        let root = tmp.join(".wm").join("wiki");
        std::fs::create_dir_all(&root).expect("create absolute root");
        let candidate = root.join("specs/x.md");
        let out = confine(&root, &candidate).expect("absolute path inside root is allowed");
        assert_eq!(out, candidate);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn absolute_outside_root_is_rejected() {
        let tmp = std::env::temp_dir().join(format!("wm-confine-abs-out-{}", std::process::id()));
        let root = tmp.join(".wm").join("wiki");
        std::fs::create_dir_all(&root).expect("create absolute root");
        let outside = tmp.join("outside/x.md");
        assert!(
            confine(&root, &outside).is_err(),
            "absolute path outside root must be rejected"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escaping_root_is_rejected() {
        use std::os::unix::fs::symlink;
        let tmp = std::env::temp_dir().join(format!("wm-confine-sym-{}", std::process::id()));
        let root = tmp.join("root");
        let outside = tmp.join("outside");
        std::fs::create_dir_all(&root).expect("create root");
        std::fs::create_dir_all(&outside).expect("create outside");
        symlink(&outside, root.join("link")).expect("create symlink");
        let res = confine(&root, Path::new("link/secret.txt"));
        assert!(
            res.is_err(),
            "symlink pointing outside the root must be rejected"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn table_driven_traversal_rejections() {
        let cases = [
            "../../etc/passwd.md",
            "../../../../../../tmp/x.md",
            "specs/../../../etc/x.md",
            "/etc/passwd",
            "tasks/../../../tasks/../secrets/x.md",
            "specs/../../wiki-secret.md",
        ];
        for case in cases {
            assert!(
                confine(Path::new(ROOT), Path::new(case)).is_err(),
                "must reject '{}'",
                case
            );
        }
    }

    #[test]
    fn table_driven_allowed_paths() {
        let cases = ["specs/my-spec.md", "tasks/not-created-yet.md", "a/b/c.md"];
        for case in cases {
            let out = confine(Path::new(ROOT), Path::new(case))
                .unwrap_or_else(|_| panic!("must allow '{}'", case));
            assert!(out.starts_with(ROOT), "'{}' must stay inside root", case);
        }
    }
}
