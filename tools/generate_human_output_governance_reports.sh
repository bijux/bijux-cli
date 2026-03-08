#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

POLICY="configs/policy/human_output_governance.json"
OUT_DIR="docs/reports/foundation"
REF_PATH="docs/reference/OPERATOR_UX_REFERENCE_GENERATED.md"
EXAMPLE_ROOT="evidence/operator/examples/human_output"
mkdir -p "$OUT_DIR" "$EXAMPLE_ROOT" "$(dirname "$REF_PATH")"

# Build concise/detailed example sets per family.
jq -c '.families[]' "$POLICY" | while IFS= read -r fam; do
  family=$(printf '%s' "$fam" | jq -r '.family')
  family_dir="$EXAMPLE_ROOT/$family"
  mkdir -p "$family_dir"

  first_file=$(printf '%s' "$fam" | jq -r '.snapshot_files[0]')
  second_file=$(printf '%s' "$fam" | jq -r '.snapshot_files[1] // .snapshot_files[0]')

  cat "$first_file" > "$family_dir/concise.txt"
  cat "$second_file" > "$family_dir/detailed.txt"
done

inv="$OUT_DIR/human_output_snapshot_inventory_report.md"
{
  echo "# Human Output Snapshot Inventory Report"
  echo
  echo "| Family | Snapshot tests | Snapshot files |"
  echo "| --- | --- | --- |"
  jq -r '.families[] | [.family, (.snapshot_tests | join(", ")), (.snapshot_files | join(", "))] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family tests files; do
        echo "| \`$family\` | \`$tests\` | \`$files\` |"
      done
} > "$inv"

missing="$OUT_DIR/human_output_surfaces_without_snapshot_report.md"
missing_rows="$(mktemp)"
jq -c '.families[]' "$POLICY" | while IFS= read -r fam; do
  family=$(printf '%s' "$fam" | jq -r '.family')
  missing_items=()

  printf '%s' "$fam" | jq -r '.snapshot_tests[]' | while IFS= read -r test_name; do
    if ! rg -q "$test_name" crates/bijux-dag-app/tests crates/bijux-dev-dag/tests -g '*.rs'; then
      echo "| \`$family\` | \`$test_name\` | \`missing snapshot test\` |" >> "$missing_rows"
    fi
  done

  printf '%s' "$fam" | jq -r '.snapshot_files[]' | while IFS= read -r snap; do
    if [[ ! -f "$snap" ]]; then
      echo "| \`$family\` | \`$snap\` | \`missing snapshot file\` |" >> "$missing_rows"
    fi
  done
done
missing_count=$(wc -l < "$missing_rows" | tr -d ' ')
{
  echo "# Human Output Surfaces Without Snapshot Report"
  echo
  echo "| Family | Surface | Gap |"
  echo "| --- | --- | --- |"
  cat "$missing_rows"
  echo
  echo "Missing human snapshot surfaces: $missing_count"
} > "$missing"
rm -f "$missing_rows"

nosnap="$OUT_DIR/human_output_without_snapshot_tests_report.md"
cp "$missing" "$nosnap"

concise_detail="$OUT_DIR/concise_detailed_human_output_coverage_report.md"
{
  echo "# Concise vs Detailed Human Output Coverage Report"
  echo
  echo "| Family | Concise example | Detailed example | Distinct |"
  echo "| --- | --- | --- | --- |"
  jq -r '.families[].family' "$POLICY" | while IFS= read -r family; do
    c="$EXAMPLE_ROOT/$family/concise.txt"
    d="$EXAMPLE_ROOT/$family/detailed.txt"
    distinct="false"
    if ! cmp -s "$c" "$d"; then
      distinct="true"
    fi
    echo "| \`$family\` | \`$c\` | \`$d\` | \`$distinct\` |"
  done
} > "$concise_detail"

drift="$OUT_DIR/wording_drift_equivalent_commands_report.md"
{
  echo "# Wording Drift Equivalent Commands Report"
  echo
  echo "| Comparison | Result |"
  echo "| --- | --- |"
  if cmp -s crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt; then
    echo "| route concise vs route detailed | \`identical\` |"
  else
    echo "| route concise vs route detailed | \`different-as-expected\` |"
  fi
  if cmp -s crates/bijux-dag-app/tests/snapshots/prove_human_output_contract.txt crates/bijux-dag-app/tests/snapshots/verify_human_output_contract.txt; then
    echo "| prove vs verify human output | \`identical\` |"
  else
    echo "| prove vs verify human output | \`different-as-expected\` |"
  fi
} > "$drift"

{
  echo "# Operator UX Reference (Generated)"
  echo
  echo "Generated from human-output snapshots and governed examples."
  echo
  jq -c '.families[]' "$POLICY" | while IFS= read -r fam; do
    family=$(printf '%s' "$fam" | jq -r '.family')
    echo "## $family"
    echo
    echo "- Snapshot tests:"
    printf '%s' "$fam" | jq -r '.snapshot_tests[]' | while IFS= read -r t; do
      echo "  - \`$t\`"
    done
    echo "- Examples:"
    echo "  - \`$EXAMPLE_ROOT/$family/concise.txt\`"
    echo "  - \`$EXAMPLE_ROOT/$family/detailed.txt\`"
    echo
  done
} > "$REF_PATH"
