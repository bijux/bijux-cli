# Inspect Diagnostics Report

Generated: 2026-03-08

## Diagnostic surfaces

- inspect behavior and shape stability:
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
  - `crates/bijux-dag-app/tests/operator_ux_contract.rs`
- corrupted and missing artifact/index handling:
  - `crates/bijux-dag-app/tests/artifact_inspect_storage_contracts.rs`
  - `crates/bijux-dag-app/tests/operator_input_no_panic_contracts.rs`
- route-level no-panic coverage:
  - `crates/bijux-dag-app/tests/route_entrypoint_no_panic_contract.rs`

## Schema and snapshot anchors

- `configs/schema/operator/run_inspect.schema.json`
- `configs/schema/operator/artifact_inspect.schema.json`
- `configs/schema/operator/run_timeline.schema.json`
- `crates/bijux-dag-app/tests/snapshots/inspect_human_output.txt`

## Current posture

- inspect diagnostics remain schema-governed, snapshot-stable, and no-panic under malformed inputs
