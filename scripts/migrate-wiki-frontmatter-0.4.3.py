#!/usr/bin/env python3
"""Migrate pre-0.4.3 wiki frontmatter to the 0.4.3 format.

Scans `.wm/wiki/**/*.md` and repairs the frontmatter corruption classes that
the 0.4.3 write-path fix eliminates (see apps/wm-core/src/page/helpers/
yaml_helper.rs and apps/wm-core/src/parser/mod.rs `inspect_frontmatter_health`):

  1. Unquoted scientific-notation `id:`  -> `id: "652e07"`
     An unquoted value like `652e07` is read by serde_yaml as a float
     (scientific notation) and silently rewritten to `6520000000.0` on the
     next whole-frontmatter round-trip. Always-quote ids going forward.
  2. Duplicate `---` frontmatter blocks  -> merge (union, preserve all fields)
     A buggy write appended a second/third `---` block instead of merging;
     this repairs to a single block keeping every field, byte-order from the
     first block.
  3. Empty / `{}` frontmatter blocks     -> rebuild from the remaining blocks
     An empty-map serialization emitted `{}` (or an empty block) — never
     persist it; drop the empty block and keep the body.
  4. Stripped fields (id/title/type)     -> restore `id` from the filename
     when the frontmatter is missing an id entirely.

Usage:
  python3 scripts/migrate-wiki-frontmatter-0.4.3.py [--apply] [--root DIR] [--dry-run]

  default  : dry-run (report only, no writes)
  --apply  : write repaired files back to disk
  --root   : project root containing .wm/wiki (default: script's repo root)
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

SCI_NOTATION_RE = re.compile(r"^[0-9]+[eE][0-9]+$")


# ---------------------------------------------------------------------------
# Detection helpers (mirror apps/wm-core/src/parser/mod.rs)
# ---------------------------------------------------------------------------

def split_frontmatter_blocks(content: str) -> list[str]:
    """Return the complete `---`-delimited YAML blocks at the top of a file."""
    blocks: list[str] = []
    rest = content.lstrip()
    while rest.startswith("---"):
        # find the closing "\n---" after the opener
        pos = rest.find("\n---", 3)
        if pos < 0:
            break
        block = rest[4:pos]  # between the opening "---\n" and the closing "\n---"
        blocks.append(block)
        rest = rest[pos + 4 :].lstrip()
    return blocks


def split_frontmatter_and_body(content: str) -> tuple[list[str], str]:
    """Split content into (frontmatter blocks, body).

    The body is everything after the LAST frontmatter block's closing marker.
    Mirrors split_frontmatter_blocks so openers/closers are handled the same
    way (a naive `find("\\n---")` would match a block opener and leak a block
    into the body).
    """
    blocks: list[str] = []
    rest = content.lstrip()
    while rest.startswith("---"):
        pos = rest.find("\n---", 3)
        if pos < 0:
            break
        blocks.append(rest[4:pos])
        rest = rest[pos + 4 :].lstrip()
    return blocks, rest


def frontmatter_id_raw(fm: str) -> str | None:
    """Raw (quote-preserving) value of a top-level `id:` line, if any."""
    for line in fm.splitlines():
        if line[:1] in (" ", "\t"):
            continue
        if ":" in line:
            key, _, value = line.partition(":")
            if key.rstrip() == "id":
                return value.strip()
    return None


def frontmatter_id(fm: str) -> str | None:
    raw = frontmatter_id_raw(fm)
    if raw is None:
        return None
    return raw.strip().strip('"').strip("'")


def looks_like_scientific_notation(value: str) -> bool:
    if not value or '"' in value or "'" in value:
        return False
    m = re.fullmatch(r"[0-9]+[eE][0-9]+", value)
    return m is not None


def parse_top_level_blocks(fm: str) -> dict[str, list[str]]:
    """Parse a frontmatter block into {top-level key: its lines} preserving order."""
    keys: dict[str, list[str]] = {}
    current: str | None = None
    for line in fm.splitlines():
        if line[:1] in (" ", "\t"):
            if current is not None:
                keys[current].append(line)
            continue
        if ":" in line:
            key, _, _ = line.partition(":")
            current = key.rstrip()
            keys.setdefault(current, []).append(line)
        else:
            current = None
    return keys


def merge_blocks(blocks: list[str]) -> str:
    """Union-merge multiple frontmatter blocks into one, preserving field order
    from the first block and appending keys missing from later blocks."""
    if not blocks:
        return ""
    merged = blocks[0].rstrip() + "\n"
    first_keys = parse_top_level_blocks(blocks[0])
    for block in blocks[1:]:
        for key, lines in parse_top_level_blocks(block).items():
            if key in first_keys:
                continue
            merged += "\n".join(lines) + "\n"
            first_keys[key] = lines
    return merged


# ---------------------------------------------------------------------------
# Repairs
# ---------------------------------------------------------------------------

def repair(content: str, filename_stem: str) -> tuple[str | None, list[str]]:
    """Return (repaired_content, list_of_changes). None if no change needed."""
    if not content.startswith("---"):
        return None, []

    blocks, body = split_frontmatter_and_body(content)
    if not blocks:
        return None, []
    body = body.lstrip()

    changes: list[str] = []
    merged = list(blocks)

    # --- Rule 3: drop empty / {} blocks, keep the rest ---------------------
    kept = []
    for i, b in enumerate(merged):
        b_trim = b.strip()
        if b_trim == "" or b_trim == "{}":
            changes.append(f"dropped empty/{{}} frontmatter block #{i + 1}")
        else:
            kept.append(b)
    merged = kept

    if not merged:
        # Nothing but empty blocks: file becomes body-only.
        repaired = body.lstrip()
        return (repaired, changes) if changes else (None, [])

    # --- Rule 2: merge duplicate blocks ------------------------------------
    if len(merged) > 1:
        changes.append(f"merged {len(merged)} frontmatter blocks into one")
        merged = [merge_blocks(merged)]

    fm = merged[0]

    # --- Rule 1: quote scientific-notation ids ------------------------------
    raw_id = frontmatter_id_raw(fm)
    if raw_id is not None and looks_like_scientific_notation(raw_id):
        changes.append(f"quoted scientific-notation id '{raw_id}'")
        fm = re.sub(
            r"^id:\s*" + re.escape(raw_id) + r"\s*$",
            f'id: "{raw_id}"',
            fm,
            count=1,
            flags=re.MULTILINE,
        )

    # --- Rule 4: restore a stripped id only when corruption was detected ----
    # The old update path stripped id/title/type from some task files. Inject
    # an id from the filename ONLY when the file shows other corruption
    # artifacts (duplicate/empty blocks) — never inject ids into pages that
    # legitimately have none (the validator flags id *mismatch*, it does not
    # require an id).
    if filename_stem and frontmatter_id(fm) is None:
        if not any(
            line[:1] not in (" ", "\t") and line.partition(":")[0].rstrip() == "id"
            for line in fm.splitlines()
        ) and changes:
            changes.append(f"restored id '{filename_stem}' from filename")
            fm = f'id: "{filename_stem}"\n' + fm

    repaired = f"---\n{fm.rstrip()}\n---\n{body}"
    return (repaired, changes) if changes else (None, [])


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="write repairs to disk")
    parser.add_argument("--dry-run", action="store_true", help="report only (default)")
    parser.add_argument("--root", type=str, default=None, help="project root (default: repo root)")
    args = parser.parse_args()

    root = Path(args.root) if args.root else Path(__file__).resolve().parent.parent
    wiki = root / ".wm" / "wiki"
    if not wiki.is_dir():
        print(f"error: no wiki at {wiki}", file=sys.stderr)
        return 1

    files = sorted(wiki.rglob("*.md"))
    changed = 0
    for path in files:
        try:
            content = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        repaired, changes = repair(content, path.stem)
        if repaired is None:
            continue
        changed += 1
        if args.apply:
            path.write_text(repaired, encoding="utf-8")
        print(f"{'[FIX]' if args.apply else '[DRY]'} {path.relative_to(root)}")
        for c in changes:
            print(f"        - {c}")

    print(f"\n{changed} file(s) would {'be repaired' if not args.apply else 'repaired'} "
          f"({len(files)} wiki files scanned)")
    if changed and not args.apply:
        print("Re-run with --apply to write changes.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
