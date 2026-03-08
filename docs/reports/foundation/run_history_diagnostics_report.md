# Run History Diagnostics Report

Generated: 2026-03-08

## Diagnostic surfaces

- schema-bound API surface report:
  - `docs/reports/foundation/run_history_api_report.json`
- damaged-run and strict verification diagnostics:
  - `crates/bijux-dag-app/tests/run_history_ancestry_contracts.rs`
  - `crates/bijux-dag-app/tests/run_history_reliability_contract.rs`
- resilience suite coverage:
  - `configs/suites/run_history_resilience_fast.json`
  - `configs/suites/run_history_many_runs_stress.json`

## Current posture

- run-history diagnostics remain schema-governed
- strict verification surfaces preserve error visibility for corrupted state
- fast and stress suites keep diagnostic paths exercised
