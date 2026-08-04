use std::path::{Component, Path, PathBuf};

use crate::error::{ToolError, ToolResult};

const ERR_ESCAPES_ROOT: &str = "Access denied: path escapes the allowed root";
const ERR_HIDDEN: &str = "Access denied: dotfiles and hidden directories are not allowed";
const DOT_PREFIX: char = '.';
const CURRENT_DIR: &str = ".";

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
    let Ok(real) = resolved.canonicalize() else {
        return false;
    };
    let Ok(real_root) = root.canonicalize() else {
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
        return Err(ToolError::invalid_params(ERR_ESCAPES_ROOT));
    }

    if symlink_escapes(&root_norm, &resolved) {
        tracing::warn!(
            "Rejected symlink escaping root: candidate={}",
            candidate.display()
        );
        return Err(ToolError::invalid_params(ERR_ESCAPES_ROOT));
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
        return Err(ToolError::invalid_params(ERR_HIDDEN));
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
}
