#!/usr/bin/env bash
set -euo pipefail

readonly EXPECTED_SINGLETON_COUNT=1
readonly SEARCH_ROOTS=(apps packages)
readonly PAGE_TYPE_MAPPING_MARKER="impl PageType"

singletons=(
  "slug|label_slug|fn label_slug\\("
  "slug|memory_slug|fn memory_slug\\("
  "slug|task_page_id_slug|fn task_page_id_slug\\("
  "frontmatter_splitter|split_first_frontmatter|fn split_first_frontmatter\\("
  "frontmatter_splitter|split_leading_frontmatter|fn split_leading_frontmatter\\("
)

count_matches() {
  grep -rnE --include='*.rs' "$1" "${SEARCH_ROOTS[@]}" | wc -l | tr -d '[:space:]'
}

violations=()

printf '%-22s %-26s %6s\n' "group" "symbol" "count"
printf '%-22s %-26s %6s\n' "-----" "------" "-----"

for entry in "${singletons[@]}"; do
  group="${entry%%|*}"
  rest="${entry#*|}"
  symbol="${rest%%|*}"
  pattern="${rest#*|}"
  count="$(count_matches "$pattern")"
  printf '%-22s %-26s %6s\n' "$group" "$symbol" "$count"
  if [ "$count" -ne "$EXPECTED_SINGLETON_COUNT" ]; then
    violations+=("$symbol: expected $EXPECTED_SINGLETON_COUNT definition(s), found $count")
  fi
done

page_type_count="$(count_matches "$PAGE_TYPE_MAPPING_MARKER")"
printf '%-22s %-26s %6s\n' "page_type_mapping" "$PAGE_TYPE_MAPPING_MARKER" "$page_type_count"
if [ "$page_type_count" -ne "$EXPECTED_SINGLETON_COUNT" ]; then
  violations+=("$PAGE_TYPE_MAPPING_MARKER: expected $EXPECTED_SINGLETON_COUNT mapping(s), found $page_type_count")
fi

echo
if [ "${#violations[@]}" -gt 0 ]; then
  echo "Gate 3 violations:"
  printf '  %s\n' "${violations[@]}"
  exit 1
fi

echo "Gate 3 passed: each distinct-named singleton helper and the PageType mapping appear exactly once."
