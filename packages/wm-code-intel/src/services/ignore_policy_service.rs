use std::ffi::OsStr;
use std::path::{Component, Path};

use ignore::WalkBuilder;

pub const VCS_DIR: &str = ".git";
pub const WIKI_DIR: &str = ".wm";

fn is_reserved_dir(name: &OsStr) -> bool {
    name == OsStr::new(VCS_DIR) || name == OsStr::new(WIKI_DIR)
}

pub fn has_reserved_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if is_reserved_dir(name)))
}

pub fn policy_builder(root: &Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .ignore(false)
        .require_git(false)
        .filter_entry(|entry| !is_reserved_dir(entry.file_name()));
    builder
}

pub fn code_walk(root: &Path) -> ignore::Walk {
    policy_builder(root).build()
}

pub fn is_index_excluded(root: &Path, path: &Path) -> bool {
    if has_reserved_component(path) {
        return true;
    }
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let mut matchers = policy_builder(root).build_matchers();
    let Some(matcher) = matchers.first_mut() else {
        return false;
    };
    matcher.matched(relative, path.is_dir()).is_ignore()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_file(root: &Path, rel: &str, content: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create parent");
        std::fs::write(path, content).expect("write file");
    }

    fn temp_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path();
        write_file(root, ".gitignore", ".env\n.env.*\ntarget/\n");
        write_file(root, ".git/config", "[core]\n");
        write_file(root, ".env", "SECRET=1\n");
        write_file(root, "target/gen.rs", "pub fn g() {}\n");
        write_file(root, "src/lib.rs", "pub fn f() {}\n");
        dir
    }

    #[test]
    fn reserved_dirs_are_always_excluded() {
        let dir = temp_project();
        let root = dir.path().canonicalize().expect("canonical root");
        assert!(has_reserved_component(&root.join(".git/config")));
        assert!(has_reserved_component(&root.join(".wm/wiki/index.md")));
        assert!(!has_reserved_component(
            &root.join(".github/workflows/a.yml")
        ));
    }

    #[test]
    fn index_policy_excludes_gitignored_and_reserved() {
        let dir = temp_project();
        let root = dir.path().canonicalize().expect("canonical root");
        assert!(is_index_excluded(&root, &root.join("target/gen.rs")));
        assert!(is_index_excluded(&root, &root.join(".env")));
        assert!(is_index_excluded(&root, &root.join(".git/config")));
        assert!(is_index_excluded(&root, &root.join(".wm/wiki/index.md")));
        assert!(!is_index_excluded(&root, &root.join("src/lib.rs")));
    }
}
