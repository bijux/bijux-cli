# Replay Planning Determinism Report

Generated: 2026-03-08

## Determinism checks

- stable plan dump ordering:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- deterministic replay-oriented plan serialization:
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs`
- replay CLI dry-run plan structure stability:
  - `crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs`

## Drift guardrails

- determinism drift diagnostics:
  - `docs/reports/foundation/determinism_drift_detection_report.md`
- replay mismatch corpus anchors:
  - `crates/bijux-dev-dag/tests/replay_mismatch_corpus_contracts.rs`

## Current posture

- replay planning determinism is guarded in both planner-core and CLI operator pathways
- drift detection reports remain present and enforced by contract suites
