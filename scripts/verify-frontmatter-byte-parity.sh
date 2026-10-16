#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
base_rev="${1:-HEAD}"
main_target="${CARGO_TARGET_DIR:-$repo_root/target}"
tmp="$(mktemp -d)"
head_tree="$tmp/head"
new_probe="$repo_root/packages/wm-engine/tests/parity_probe.rs"

cleanup() {
  rm -f "$new_probe"
  git -C "$repo_root" worktree remove --force "$head_tree" >/dev/null 2>&1 || true
  git -C "$repo_root" worktree prune >/dev/null 2>&1 || true
  rm -rf "$tmp"
}
trap cleanup EXIT

git -C "$repo_root" worktree add --detach "$head_tree" "$base_rev" >/dev/null

PROBE=$(cat <<'RUST'
use std::path::PathBuf;

use __CRATE__::parser::{extract_frontmatter, frontmatter_to_yaml};

#[test]
fn dump() {
    let dir = PathBuf::from(std::env::var("WM_PARITY_TASKS_DIR").expect("WM_PARITY_TASKS_DIR"));
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tasks dir")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    paths.sort();
    for path in paths {
        let content = std::fs::read_to_string(&path).expect("read");
        let fm = match extract_frontmatter(&content).0 {
            Some(fm) => fm,
            None => continue,
        };
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let first = frontmatter_to_yaml(&fm);
        let wrapped = format!("---\n{}---\n\nbody\n", first);
        let idempotent = match extract_frontmatter(&wrapped).0 {
            Some(again) => first == frontmatter_to_yaml(&again),
            None => false,
        };
        println!("<<<PAGE {}>>>", name);
        print!("{}", first);
        println!("<<<IDEM {} {}>>>", name, idempotent);
        println!("<<<END {}>>>", name);
    }
}
RUST
)

printf '%s\n' "$PROBE" | sed 's/__CRATE__/wm_core/' > "$head_tree/apps/wm-core/tests/parity_probe.rs"
printf '%s\n' "$PROBE" | sed 's/__CRATE__/wm_engine/' > "$new_probe"

export WM_PARITY_TASKS_DIR="$repo_root/.wm/wiki/tasks"

( cd "$head_tree" && CARGO_TARGET_DIR="$main_target" cargo test -p wm-core --test parity_probe -- --nocapture 2>/dev/null ) > "$tmp/old.raw"
( cd "$repo_root" && cargo test -p wm-engine --test parity_probe -- --nocapture 2>/dev/null ) > "$tmp/new.raw"

awk '/^<<<PAGE /{p=1} p{print} /^<<<IDEM /{p=0}' "$tmp/old.raw" > "$tmp/old.emit"
awk '/^<<<PAGE /{p=1} p{print} /^<<<IDEM /{p=0}' "$tmp/new.raw" > "$tmp/new.emit"
grep '^<<<IDEM ' "$tmp/old.raw" | sed 's/^<<<IDEM /<<</; s/>>>.*/>>>/' > "$tmp/old.idem" || true
grep '^<<<IDEM ' "$tmp/new.raw" | sed 's/^<<<IDEM /<<</; s/>>>.*/>>>/' > "$tmp/new.idem" || true

echo "pages: old=$(grep -c '^<<<PAGE ' "$tmp/old.emit" || true) new=$(grep -c '^<<<PAGE ' "$tmp/new.emit" || true)"
if diff "$tmp/old.emit" "$tmp/new.emit"; then
  echo "PARITY OK: new serializer output == ${base_rev} serializer output on every page"
fi
echo "non-idempotent pages: old=$(grep -c 'false' "$tmp/old.idem" || true) new=$(grep -c 'false' "$tmp/new.idem" || true)"
echo "idempotence flags agree between revisions: $(cmp -s "$tmp/old.idem" "$tmp/new.idem" && echo yes || echo no)"
