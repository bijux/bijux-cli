#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

POLICY="configs/policy/fixture_family_governance.json"
OUT_DIR="docs/reports/foundation"
mkdir -p "$OUT_DIR"

family_names=$(jq -r '.governed_families[].family' "$POLICY")

collect_files_for_family() {
  local family="$1"
  jq -r --arg family "$family" '.governed_families[] | select(.family==$family) | .roots[]' "$POLICY" | while IFS= read -r root; do
    if [[ -f "$root" ]]; then
      printf '%s\n' "$root"
    elif [[ -d "$root" ]]; then
      rg --files "$root"
    fi
  done | sort -u
}

for family in $family_names; do
  report="$OUT_DIR/${family}_fixture_inventory_report.md"
  owner_suite=$(jq -r --arg family "$family" '.governed_families[] | select(.family==$family) | .owner_suite' "$POLICY")
  owner_crate=$(jq -r --arg family "$family" '.governed_families[] | select(.family==$family) | .owner_crate' "$POLICY")
  purpose=$(jq -r --arg family "$family" '.governed_families[] | select(.family==$family) | .fixture_purpose' "$POLICY")

  family_title="$(printf '%s' "$family" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')"
  {
    echo "# ${family_title} Fixture Inventory Report"
    echo
    echo "- Purpose: $purpose"
    echo "- Owner suite: $owner_suite"
    echo "- Owner crate: $owner_crate"
    echo
    echo "| Fixture path | Owner suite | Owner crate |"
    echo "| --- | --- | --- |"

    file_count=0
    while IFS= read -r file; do
      [[ -z "$file" ]] && continue
      file_count=$((file_count + 1))
      echo "| \`$file\` | \`$owner_suite\` | \`$owner_crate\` |"
    done < <(collect_files_for_family "$family")

    echo
    echo "Total fixtures: $file_count"
  } > "$report"
done

missing_report="$OUT_DIR/fixture_governance_missing_owner_report.md"
{
  echo "# Fixture Governance Missing Owner Report"
  echo
  echo "| Family | Missing owner suite | Missing owner crate |"
  echo "| --- | --- | --- |"
  jq -r '.governed_families[] | [.family, (.owner_suite == null or .owner_suite == ""), (.owner_crate == null or .owner_crate == "")] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family missing_suite missing_crate; do
        echo "| \`$family\` | \`$missing_suite\` | \`$missing_crate\` |"
      done
} > "$missing_report"

missing_suite_report="$OUT_DIR/fixtures_with_no_owning_suite_report.md"
{
  echo "# Fixtures With No Owning Suite Report"
  echo
  echo "| Family | Owner suite |"
  echo "| --- | --- |"
  jq -r '.governed_families[] | select(.owner_suite == null or .owner_suite == "") | [.family, .owner_suite] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family owner_suite; do
        echo "| \`$family\` | \`$owner_suite\` |"
      done
} > "$missing_suite_report"

missing_crate_report="$OUT_DIR/fixtures_with_no_owning_crate_report.md"
{
  echo "# Fixtures With No Owning Crate Report"
  echo
  echo "| Family | Owner crate |"
  echo "| --- | --- |"
  jq -r '.governed_families[] | select(.owner_crate == null or .owner_crate == "") | [.family, .owner_crate] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family owner_crate; do
        echo "| \`$family\` | \`$owner_crate\` |"
      done
} > "$missing_crate_report"

unreferenced_report="$OUT_DIR/unreferenced_fixtures_report.md"
{
  echo "# Unreferenced Fixtures Report"
  echo
  echo "| Family | Fixture path |"
  echo "| --- | --- |"

  for family in $family_names; do
    while IFS= read -r fixture; do
      [[ -z "$fixture" ]] && continue
      ref_count="$( (rg -F --glob '!docs/reports/**' --glob '!target/**' --glob '!artifacts/**' --glob '!evidence/reports/**' --glob '!docs/adr/**' -- "$fixture" crates configs tools Makefile justfile .github 2>/dev/null || true) | wc -l | tr -d ' ' )"
      if [[ "$ref_count" -eq 0 ]]; then
        echo "| \`$family\` | \`$fixture\` |"
      fi
    done < <(collect_files_for_family "$family")
  done
} > "$unreferenced_report"

duplicate_report="$OUT_DIR/duplicate_fixtures_semantic_hash_report.md"
tmp_hash_file="$(mktemp)"
for family in $family_names; do
  while IFS= read -r fixture; do
    [[ -z "$fixture" ]] && continue
    if [[ -f "$fixture" ]]; then
      hash=$(shasum -a 256 "$fixture" | awk '{print $1}')
      printf '%s\t%s\n' "$hash" "$fixture" >> "$tmp_hash_file"
    fi
  done < <(collect_files_for_family "$family")
done

{
  echo "# Duplicate Fixtures Semantic Hash Report"
  echo
  echo "| SHA-256 | Fixture paths |"
  echo "| --- | --- |"
  cut -f1 "$tmp_hash_file" | sort | uniq -d | while IFS= read -r dup_hash; do
    unique_count=$(awk -F '\t' -v h="$dup_hash" '$1 == h { print $2 }' "$tmp_hash_file" | sort -u | wc -l | tr -d ' ')
    if [[ "$unique_count" -gt 1 ]]; then
      joined=$(awk -F '\t' -v h="$dup_hash" '$1 == h { print $2 }' "$tmp_hash_file" | sort -u | sed 's/^/`/;s/$/`/' | awk 'NR==1{printf "%s",$0} NR>1{printf "<br>%s",$0} END{print ""}')
      echo "| \`$dup_hash\` | $joined |"
    fi
  done
} > "$duplicate_report"
rm -f "$tmp_hash_file"

stale_report="$OUT_DIR/stale_fixture_schema_field_report.md"
{
  echo "# Stale Fixture Schema Field Report"
  echo
  echo "| Fixture path | Legacy field pattern |"
  echo "| --- | --- |"
  jq -r '.legacy_schema_field_patterns[]' "$POLICY" | while IFS= read -r pattern; do
    for family in $family_names; do
      while IFS= read -r fixture; do
        [[ -z "$fixture" ]] && continue
        [[ ! -f "$fixture" ]] && continue
        if rg -q -F -- "$pattern" "$fixture"; then
          echo "| \`$fixture\` | \`$pattern\` |"
        fi
      done < <(collect_files_for_family "$family")
    done
  done | sort -u
} > "$stale_report"

quick_ref="docs/reference/FIXTURE_GOVERNANCE_QUICK_REFERENCE.md"
{
  echo "# Fixture Governance Quick Reference"
  echo
  echo "Policy source: \`configs/policy/fixture_family_governance.json\`"
  echo
  echo "## Governed families"
  echo
  echo "| Family | Purpose | Owner | Lane | Taxonomy |"
  echo "| --- | --- | --- | --- | --- |"
  jq -r '.governed_families[] | [.family, .fixture_purpose, .fixture_owner, .fixture_lane, .fixture_taxonomy] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family purpose owner lane taxonomy; do
        echo "| \`$family\` | $purpose | \`$owner\` | \`$lane\` | \`$taxonomy\` |"
      done
  echo
  echo "## Generated reports"
  echo
  for family in $family_names; do
    echo "- \`docs/reports/foundation/${family}_fixture_inventory_report.md\`"
  done
  echo "- \`docs/reports/foundation/fixture_governance_missing_owner_report.md\`"
  echo "- \`docs/reports/foundation/fixtures_with_no_owning_suite_report.md\`"
  echo "- \`docs/reports/foundation/fixtures_with_no_owning_crate_report.md\`"
  echo "- \`docs/reports/foundation/unreferenced_fixtures_report.md\`"
  echo "- \`docs/reports/foundation/duplicate_fixtures_semantic_hash_report.md\`"
  echo "- \`docs/reports/foundation/stale_fixture_schema_field_report.md\`"
} > "$quick_ref"
