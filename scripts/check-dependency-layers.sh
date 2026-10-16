#!/usr/bin/env bash
set -euo pipefail

readonly METADATA_FORMAT="1"

metadata_file="$(mktemp)"
trap 'rm -f "$metadata_file"' EXIT

cargo metadata --no-deps --format-version "$METADATA_FORMAT" > "$metadata_file"

python3 - "$metadata_file" <<'PY'
import json
import sys

LAYER_POLICY = {
    "rmcp": {"wm-cli"},
    "clap": {"wm-cli"},
    "notify": {"wm-cli", "wm-core"},
    "tracing-subscriber": {"wm-cli", "wm-core"},
    "tracing-appender": {"wm-cli", "wm-core"},
    "rust-embed": {"wm-cli", "wm-core"},
}

with open(sys.argv[1]) as handle:
    metadata = json.load(handle)

violations = []
for package in metadata["packages"]:
    declared = {dependency["name"] for dependency in package["dependencies"]}
    for dependency, owners in LAYER_POLICY.items():
        if dependency in declared and package["name"] not in owners:
            allowed = ", ".join(sorted(owners))
            violations.append(
                "{} declares '{}' (allowed only by: {})".format(
                    package["name"], dependency, allowed
                )
            )

if violations:
    print("Gate 2 violations:")
    for violation in violations:
        print("  " + violation)
    sys.exit(1)

print("Gate 2 passed: deny-listed dependencies respect the layer policy.")
PY
