# Replay Planning Status Report (201-220)

Generated: 2026-03-08

This report maps tasks 201-220 to replay planning tests, fixtures, schemas,
diagnostics, dashboards, and architectural guarantees.

## 201-206 replay planning path and stability checks

- imported-run replay path and lineage:
  - `crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs`
- plan stability and deterministic lowering:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
  - `crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs`

## 207-208 replay drift fixtures and determinism checks

- replay-oriented fixture set:
  - `crates/bijux-dag-core/tests/snapshots/replay_oriented.dag.json`
  - `crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json`
  - `crates/bijux-dag-core/tests/snapshots/imported_bundle_replay.dag.json`
- determinism coverage:
  - `crates/bijux-dag-runtime/tests/replay_determinism_fuzz_contracts.rs`
  - `docs/reports/foundation/REPLAY_PLANNING_DETERMINISM_REPORT.md`

## 209-215 explain, schema, failure, and corruption behavior

- explain and output snapshots:
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
  - `crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs`
- schema and planner diagnostics:
  - `crates/bijux-dag-core/tests/planner_error_and_schema_contracts.rs`
  - `configs/schema/execution_plan.schema.json`
  - `configs/schema/planner_explain.schema.json`
- replay hardening behavior coverage:
  - `crates/bijux-dev-dag/tests/replay_hardening_contracts.rs`
  - `crates/bijux-dev-dag/tests/replay_fidelity_contracts.rs`
  - `crates/bijux-dev-dag/tests/replay_mismatch_corpus_contracts.rs`

## 216-217 complexity and determinism reports

- `docs/reports/foundation/REPLAY_PLANNING_COMPLEXITY_REPORT.md`
- `docs/reports/foundation/REPLAY_PLANNING_DETERMINISM_REPORT.md`

## 218 replay planning invariants verification suite

- `configs/suites/replay_planning_invariants.json`
- `crates/bijux-dev-dag/tests/replay_hardening_contracts.rs`

## 219 replay plan consistency dashboard

- `docs/reports/foundation/REPLAY_PLAN_CONSISTENCY_DASHBOARD.md`

## 220 ADR

- `docs/adr/20260308-REPLAY-PLANNING-GUARANTEES.md`
