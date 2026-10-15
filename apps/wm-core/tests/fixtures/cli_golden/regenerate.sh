#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$REPO_ROOT"
cargo test -p wm-core --test cli_characterization_test -- --ignored --exact regenerate_goldens
