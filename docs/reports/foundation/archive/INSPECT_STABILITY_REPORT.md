# Inspect Stability Report

Generated: 2026-03-08

## Stability signals

- inspect output snapshot stability:
  - `crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs`
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
- imported/replay/partial/corrupt run behavior:
  - `crates/bijux-dag-app/tests/operator_ux_contract.rs`
  - `crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs`
- lineage-focused inspect coverage:
  - `docs/reports/foundation/artifact_lineage_visualization_report.md`
  - `crates/bijux-dev-dag/tests/artifact_lineage_completion_contracts.rs`

## Performance anchors

- `docs/reports/foundation/app_inspect_explain_latency_baseline.md`
- `docs/reports/foundation/artifact_inspect_verify_latency_report.md`

## Current posture

- inspect surfaces remain stable across route, schema, and snapshot contracts
- lineage and latency signals are visible in generated diagnostics reports
