# Run History Consistency Dashboard

Generated: 2026-03-08

## Coverage signals

- ordering and deterministic traversal:
  - `crates/bijux-dag-app/tests/run_history_reliability_contract.rs`
  - `crates/bijux-dag-app/tests/run_history_ancestry_contracts.rs`
- schema and identity lockstep:
  - `crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs`
  - `crates/bijux-dev-dag/tests/run_history_api_report_contracts.rs`
- corruption resilience and salvage:
  - `crates/bijux-dag-app/tests/run_history_hardening_contract.rs`
  - `docs/reports/foundation/RUN_HISTORY_CORRUPTION_RESILIENCE_REPORT.md`

## Operational signals

- size-growth tracking:
  - `docs/reports/foundation/RUN_HISTORY_SIZE_GROWTH_REPORT.md`
- diagnostics tracking:
  - `docs/reports/foundation/RUN_HISTORY_DIAGNOSTICS_REPORT.md`
- performance anchors:
  - `docs/reports/foundation/run_history_query_latency_report.md`

## Current status

- run-history invariants: covered
- corruption resilience: covered
- schema and diagnostics consistency: covered
