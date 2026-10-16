#!/usr/bin/env bash
set -euo pipefail

readonly DEFAULT_PACKAGE_FLOOR=1500
readonly DEFAULT_CORE_CEILING=13000
readonly WAIVER_FILE=".github/workspace-crates.toml"
readonly PACKAGES_DIR="packages"
readonly CORE_DIR="apps/wm-core"
readonly SRC_DIR="src"

count_source_lines() {
  find "$1/$SRC_DIR" -type f -name '*.rs' -exec cat {} + | wc -l | tr -d '[:space:]'
}

waiver_reason() {
  awk -v want="$1" '
    $0 ~ "^\\[waivers\\." want "\\]" { in_section = 1; next }
    /^\[/ { in_section = 0 }
    in_section && /^[[:space:]]*reason[[:space:]]*=/ {
      sub(/^[[:space:]]*reason[[:space:]]*=[[:space:]]*/, "")
      gsub(/^"|"$/, "")
      print
      exit
    }
  ' "$WAIVER_FILE"
}

package_floor="$DEFAULT_PACKAGE_FLOOR"
core_ceiling="$DEFAULT_CORE_CEILING"

if [ -f "$WAIVER_FILE" ]; then
  parsed_floor="$(sed -n 's/^[[:space:]]*package_line_floor[[:space:]]*=[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$WAIVER_FILE" | head -n 1)"
  parsed_ceiling="$(sed -n 's/^[[:space:]]*wm_core_line_ceiling[[:space:]]*=[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$WAIVER_FILE" | head -n 1)"
  if [ -n "$parsed_floor" ]; then
    package_floor="$parsed_floor"
  fi
  if [ -n "$parsed_ceiling" ]; then
    core_ceiling="$parsed_ceiling"
  fi
fi

violations=()

printf '%-24s %8s  %s\n' "crate" "lines" "status"
printf '%-24s %8s  %s\n' "-----" "-----" "------"

for dir in "$PACKAGES_DIR"/*/; do
  if [ ! -d "$dir" ]; then
    continue
  fi
  name="$(basename "$dir")"
  lines="$(count_source_lines "$dir")"
  if [ "$lines" -ge "$package_floor" ]; then
    status="ok (>= floor $package_floor)"
  else
    reason="$(waiver_reason "$name")"
    if [ -n "$reason" ]; then
      status="waived: $reason"
    else
      status="VIOLATION: below floor $package_floor and not waived"
      violations+=("packages/$name: $lines lines < floor $package_floor and no waiver in $WAIVER_FILE")
    fi
  fi
  printf '%-24s %8s  %s\n' "$name" "$lines" "$status"
done

core_lines="$(count_source_lines "$CORE_DIR")"
if [ "$core_lines" -le "$core_ceiling" ]; then
  core_status="ok (<= ceiling $core_ceiling)"
else
  core_reason="$(waiver_reason "wm-core")"
  if [ -n "$core_reason" ]; then
    core_status="waived: $core_reason"
  else
    core_status="VIOLATION: above ceiling $core_ceiling and not waived"
    violations+=("$CORE_DIR: $core_lines lines > ceiling $core_ceiling and no waiver in $WAIVER_FILE")
  fi
fi
printf '%-24s %8s  %s\n' "wm-core (ceiling)" "$core_lines" "$core_status"

echo
if [ "${#violations[@]}" -gt 0 ]; then
  echo "Gate 1 violations:"
  printf '  %s\n' "${violations[@]}"
  exit 1
fi

echo "Gate 1 passed: every packages/* crate clears the floor or is waived, and wm-core is within its ceiling."
