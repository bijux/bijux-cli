# Explain Coverage Report

Generated: 2026-03-08

## Coverage surfaces

- explain behavior and wording contracts:
  - `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`
- advanced explainability corpus and regression governance:
  - `crates/bijux-dev-dag/tests/advanced_explainability_completion_contracts.rs`
- app-level explain-oriented contracts:
  - `crates/bijux-dag-app/tests/diff_explain_contract.rs`
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
  - `crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs`

## Schema and snapshot anchors

- `configs/schema/operator/run_explain_failure.schema.json`
- `configs/schema/operator/run_id_explain.schema.json`
- `crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt`
- `crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt`

## Current posture

- explain coverage spans schema validation, snapshots, drift classification, and regression corpora
