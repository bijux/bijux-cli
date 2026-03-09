# Replay Plan Consistency Dashboard

Generated: 2026-03-08

## Coverage signals

- replay lineage planning contracts:
  - `crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs`
- planner replay fixture contracts:
  - `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`
- replay hardening and equivalence governance:
  - `crates/bijux-dev-dag/tests/replay_hardening_contracts.rs`
  - `crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs`

## Stability signals

- replay planning complexity:
  - `docs/reports/foundation/REPLAY_PLANNING_COMPLEXITY_REPORT.md`
- replay planning determinism:
  - `docs/reports/foundation/REPLAY_PLANNING_DETERMINISM_REPORT.md`
- replay benchmark and regression coverage:
  - `docs/reports/foundation/replay_equivalence_benchmarks_report.md`

## Current status

- replay planning invariants: covered
- imported-run replay compatibility: covered
- replay determinism and drift visibility: covered
