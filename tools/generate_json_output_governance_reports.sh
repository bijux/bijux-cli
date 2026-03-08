#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

POLICY="configs/policy/json_output_governance.json"
OUT_DIR="docs/reports/foundation"
REF_DIR="docs/reference"
EXAMPLE_ROOT="evidence/operator/examples/stable_json"
mkdir -p "$OUT_DIR" "$REF_DIR" "$EXAMPLE_ROOT"

schema_to_example_dir() {
  local schema_rel="$1"
  local stem
  stem="$(basename "$schema_rel" .json)"
  echo "$EXAMPLE_ROOT/$stem"
}

# Generate minimal/maximal examples for every schema.
jq -r '.stable_command_families[].schemas[]' "$POLICY" | sort -u | while IFS= read -r schema; do
  dir="$(schema_to_example_dir "$schema")"
  mkdir -p "$dir"

  jq '{
      schema: input_filename,
      example_type: "minimal",
      data: ((.required // []) | reduce .[] as $k ({}; .[$k] = "example"))
    }' "$schema" > "$dir/minimal.json"

  jq '{
      schema: input_filename,
      example_type: "maximal",
      data: (((.properties // {}) | keys) | reduce .[] as $k ({}; .[$k] = "example"))
    }' "$schema" > "$dir/maximal.json"
done

# 641: command -> schema inventory
inv_a="$OUT_DIR/json_command_schema_inventory_report.md"
{
  echo "# JSON Command to Schema Inventory Report"
  echo
  echo "| Family | Command | Schema |"
  echo "| --- | --- | --- |"
  jq -r '.stable_command_families[] | .family as $f | .commands[] as $c | .schemas[] | [$f,$c,.] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family command schema; do
        echo "| \`$family\` | \`$command\` | \`$schema\` |"
      done
} > "$inv_a"

# 642: schema -> command/tests inventory
inv_b="$OUT_DIR/schema_command_test_inventory_report.md"
{
  echo "# Schema to Command and Test Inventory Report"
  echo
  echo "| Schema | Family | Commands | Lockstep markers |"
  echo "| --- | --- | --- | --- |"

  jq -c '.stable_command_families[]' "$POLICY" | while IFS= read -r family_obj; do
    family=$(printf '%s' "$family_obj" | jq -r '.family')
    commands=$(printf '%s' "$family_obj" | jq -r '.commands | join(", ")')
    markers=$(printf '%s' "$family_obj" | jq -r '.lockstep_markers | join(", ")')
    printf '%s' "$family_obj" | jq -r '.schemas[]' | while IFS= read -r schema; do
      echo "| \`$schema\` | \`$family\` | \`$commands\` | \`$markers\` |"
    done
  done
} > "$inv_b"

# 653: schemas with no example output
missing_examples="$OUT_DIR/schema_without_example_output_report.md"
missing_examples_rows="$(mktemp)"
jq -r '.stable_command_families[].schemas[]' "$POLICY" | sort -u | while IFS= read -r schema; do
  dir="$(schema_to_example_dir "$schema")"
  min="false"
  max="false"
  [[ -f "$dir/minimal.json" ]] || min="true"
  [[ -f "$dir/maximal.json" ]] || max="true"
  if [[ "$min" == "true" || "$max" == "true" ]]; then
    echo "| \`$schema\` | \`$min\` | \`$max\` |" >> "$missing_examples_rows"
  fi
done
missing_examples_count="$(wc -l < "$missing_examples_rows" | tr -d ' ')"
{
  echo "# Schemas Without Example Output Report"
  echo
  echo "| Schema | Missing minimal | Missing maximal |"
  echo "| --- | --- | --- |"
  cat "$missing_examples_rows"
  echo
  echo "Missing schema examples: $missing_examples_count"
} > "$missing_examples"
rm -f "$missing_examples_rows"

# 654: commands with json output but no lockstep marker
missing_lockstep="$OUT_DIR/commands_without_json_lockstep_report.md"
missing_lockstep_rows="$(mktemp)"
jq -c '.stable_command_families[]' "$POLICY" | while IFS= read -r family_obj; do
  family=$(printf '%s' "$family_obj" | jq -r '.family')
  count=$(printf '%s' "$family_obj" | jq -r '.commands | length')
  i=0
  while [[ "$i" -lt "$count" ]]; do
    cmd=$(printf '%s' "$family_obj" | jq -r ".commands[$i]")
    marker=$(printf '%s' "$family_obj" | jq -r ".lockstep_markers[$i] // \"\"")
    if [[ -z "$marker" ]] || ! rg -q "$marker" crates/bijux-dag-app/tests crates/bijux-dev-dag/tests -g '*.rs'; then
      echo "| \`$family\` | \`$cmd\` | \`$marker\` |" >> "$missing_lockstep_rows"
    fi
    i=$((i + 1))
  done
done
missing_lockstep_count="$(wc -l < "$missing_lockstep_rows" | tr -d ' ')"
{
  echo "# Commands Without JSON Lockstep Report"
  echo
  echo "| Family | Command | Missing lockstep marker |"
  echo "| --- | --- | --- |"
  cat "$missing_lockstep_rows"
  echo
  echo "Commands missing lockstep tests: $missing_lockstep_count"
} > "$missing_lockstep"
rm -f "$missing_lockstep_rows"

# 657: schema registry page
schema_registry="$REF_DIR/SCHEMA_REGISTRY.md"
{
  echo "# Schema Registry"
  echo
  echo "Generated from \`configs/policy/json_output_governance.json\`."
  echo
  echo "| Schema | Example directory |"
  echo "| --- | --- |"
  jq -r '.stable_command_families[].schemas[]' "$POLICY" | sort -u | while IFS= read -r schema; do
    dir="$(schema_to_example_dir "$schema")"
    echo "| \`$schema\` | \`$dir\` |"
  done
} > "$schema_registry"

# 658: stable output command registry page
command_registry="$REF_DIR/STABLE_JSON_OUTPUT_COMMAND_REGISTRY.md"
{
  echo "# Stable JSON Output Command Registry"
  echo
  echo "Generated from \`configs/policy/json_output_governance.json\`."
  echo
  echo "| Family | Commands |"
  echo "| --- | --- |"
  jq -r '.stable_command_families[] | [.family, (.commands | join(", "))] | @tsv' "$POLICY" \
    | while IFS=$'\t' read -r family commands; do
        echo "| \`$family\` | \`$commands\` |"
      done
} > "$command_registry"
