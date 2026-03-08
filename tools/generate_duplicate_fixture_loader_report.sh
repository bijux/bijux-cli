#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

OUT="docs/reports/foundation/duplicate_fixture_loader_helpers_report.md"
PATTERN='^\s*fn\s+(load_[a-zA-Z0-9_]*fixture|read_[a-zA-Z0-9_]*fixture|fixture_path|fixture_dir|fixtures_root|parse_fixture)\b'

matches=$(rg -n "$PATTERN" crates/bijux-dag-app crates/bijux-dag-runtime crates/bijux-dag-artifacts crates/bijux-dev-dag --glob '*.rs' || true)

declare_tmp="$(mktemp)"
if [[ -n "$matches" ]]; then
  printf '%s\n' "$matches" > "$declare_tmp"
else
  : > "$declare_tmp"
fi

{
  echo "# Duplicate Fixture Loader Helpers Report"
  echo
  echo "Generated from fixture loader helper function signatures in app/runtime/artifacts/dev-dag crates."
  echo
  echo "| Helper name | Occurrences | Locations |"
  echo "| --- | --- | --- |"

  if [[ -s "$declare_tmp" ]]; then
    sed -E 's#^([^:]+):([0-9]+):.*fn[[:space:]]+([a-zA-Z0-9_]+).*#\3\t\1:\2#' "$declare_tmp" \
      | sort \
      | awk -F '\t' '
          {
            count[$1]++;
            if (loc[$1] == "") {
              loc[$1]=$2;
            } else {
              loc[$1]=loc[$1]"<br>"$2;
            }
          }
          END {
            for (k in count) {
              printf("| `%s` | %d | %s |\n", k, count[k], loc[k]);
            }
          }
        ' | sort
  fi

  duplicate_count=0
  if [[ -s "$declare_tmp" ]]; then
    duplicate_count=$(sed -E 's#^([^:]+):([0-9]+):.*fn[[:space:]]+([a-zA-Z0-9_]+).*#\3#' "$declare_tmp" | sort | uniq -d | wc -l | tr -d ' ')
  fi
  echo
  echo "Duplicate helper names: $duplicate_count"
} > "$OUT"

rm -f "$declare_tmp"
