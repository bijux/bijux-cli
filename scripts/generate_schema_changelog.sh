#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
out="$root/docs/reports/foundation/schema_changelog.md"

{
  echo "# Schema Changelog"
  echo
  echo "Generated from files under \`configs/schema\`."
  echo
  echo "## Schemas"
  find "$root/configs/schema" -type f -name "*.json" | sort | while read -r f; do
    rel="${f#$root/}"
    sum="$(shasum -a 256 "$f" | awk '{print $1}')"
    echo "- $rel :: $sum"
  done
} > "$out"

echo "wrote $out"
