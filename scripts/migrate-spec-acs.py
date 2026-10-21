#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

SPEC_DIR = Path(".wm/wiki/specs")
CLI = Path("target/debug/wm-cli")
BOILERPLATE = "Spec reviewed and its requirements are implemented"
CHECKBOX = re.compile(r"^\s*[-*+]\s+\[([ xX])\]\s+(.+?)\s*$")
HEADING = re.compile(r"^\s*#{1,6}\s+(.*)$")
EXCLUDED_SECTIONS = (
    "open question",
    "risk",
    "out of scope",
    "non-goal",
    "reference",
    "related",
)
FRONTMATTER_ID = re.compile(r'^id:\s*"?([^"\n]+)"?\s*$', re.MULTILINE)


def split_frontmatter(text):
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---", 4)
    if end < 0:
        return None, text
    return text[4:end], text[end + 4 :]


def section_filter(body):
    titles = [match.group(1).lower() for match in map(HEADING.match, body.splitlines()) if match]
    return any("acceptance" in title for title in titles)


def collect_acceptance_criteria(body):
    items = []
    in_fence = False
    acceptance_only = section_filter(body)
    collecting = not acceptance_only
    for line in body.splitlines():
        stripped = line.strip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        heading = HEADING.match(line)
        if heading:
            title = heading.group(1).lower()
            if acceptance_only:
                collecting = "acceptance" in title
                continue
            collecting = not any(marker in title for marker in EXCLUDED_SECTIONS)
            continue
        if not collecting:
            continue
        match = CHECKBOX.match(line)
        if not match:
            continue
        items.append(
            {
                "text": match.group(2).strip(),
                "checked": match.group(1).lower() == "x",
            }
        )
    return items


def page_id(frontmatter, path):
    match = FRONTMATTER_ID.search(frontmatter)
    if match:
        return match.group(1).strip()
    return f"wiki:specs:{path.stem}"


def strip_block(frontmatter, key):
    out = []
    skipping = False
    for line in frontmatter.splitlines():
        if skipping:
            if line.startswith((" ", "\t")) or not line.strip() or line.startswith(("- ", "* ")):
                continue
            skipping = False
        if line.startswith(key + ":"):
            skipping = True
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def rollback(apply, report_path):
    report = json.loads(Path(report_path).read_text(encoding="utf-8"))
    removed = 0
    for entry in report.get("pages", []):
        if entry.get("status") != "updated":
            continue
        path = Path(entry["path"])
        text = path.read_text(encoding="utf-8")
        frontmatter, body = split_frontmatter(text)
        if frontmatter is None:
            continue
        stripped = strip_block(frontmatter, "acceptance_criteria")
        if stripped != frontmatter:
            removed += 1
            if apply:
                path.write_text("---\n" + stripped + "---" + body, encoding="utf-8")
    print(json.dumps({"rollback": True, "apply": apply, "pages": removed}))


def update_page(identifier, criteria):
    payload = json.dumps(
        {"extra_frontmatter": {"acceptance_criteria": criteria}}, ensure_ascii=False
    )
    return subprocess.run(
        [str(CLI), "page", "update", identifier],
        input=payload,
        text=True,
        capture_output=True,
        check=False,
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--limit", type=int, default=0)
    parser.add_argument("--report", default=".wm/migrations/spec-acs-report.json")
    parser.add_argument("--rollback", action="store_true")
    args = parser.parse_args()

    if args.rollback:
        rollback(args.apply, args.report)
        return

    paths = sorted(SPEC_DIR.glob("*.md"))
    report = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "apply": args.apply,
        "total": len(paths),
        "already": 0,
        "from_checkboxes": 0,
        "boilerplate": 0,
        "failed": 0,
        "pages": [],
    }
    todo = []
    for path in paths:
        text = path.read_text(encoding="utf-8")
        frontmatter, body = split_frontmatter(text)
        if frontmatter is None:
            report["failed"] += 1
            report["pages"].append({"path": str(path), "status": "no-frontmatter"})
            continue
        if re.search(r"^acceptance_criteria:", frontmatter, re.MULTILINE):
            report["already"] += 1
            continue
        criteria = collect_acceptance_criteria(body)
        source = "checkboxes"
        if not criteria:
            criteria = [{"text": BOILERPLATE, "checked": False}]
            source = "boilerplate"
        todo.append(
            {
                "path": str(path),
                "id": page_id(frontmatter, path),
                "source": source,
                "criteria": criteria,
            }
        )

    if args.limit:
        todo = todo[: args.limit]

    for entry in todo:
        if entry["source"] == "checkboxes":
            report["from_checkboxes"] += 1
        else:
            report["boilerplate"] += 1
        if not args.apply:
            report["pages"].append(
                {
                    "id": entry["id"],
                    "path": entry["path"],
                    "source": entry["source"],
                    "criteria": len(entry["criteria"]),
                    "status": "dry-run",
                }
            )
            continue
        result = update_page(entry["id"], entry["criteria"])
        status = "updated" if result.returncode == 0 else "failed"
        if result.returncode != 0:
            report["failed"] += 1
        report["pages"].append(
            {
                "id": entry["id"],
                "path": entry["path"],
                "source": entry["source"],
                "criteria": len(entry["criteria"]),
                "status": status,
                "error": result.stderr.strip()[:500] if result.returncode else "",
            }
        )

    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(
        json.dumps(
            {k: report[k] for k in ("total", "already", "from_checkboxes", "boilerplate", "failed")}
        )
    )
    if report["failed"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
