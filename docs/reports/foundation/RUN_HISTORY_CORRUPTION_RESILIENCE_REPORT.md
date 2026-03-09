# Run History Corruption Resilience Report

Generated: 2026-03-08

## Corruption and recovery surfaces

- corrupt manifest salvage behavior:
  - `crates/bijux-dag-app/tests/run_history_hardening_contract.rs`
  - `crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs`
- missing trace and observability diagnostics:
  - `crates/bijux-dag-app/tests/run_history_reliability_contract.rs`
  - `crates/bijux-dag-app/tests/run_history_ancestry_contracts.rs`
- run manifest regression corpus:
  - `evidence/cache/replay/run_manifest_regression_corpus.json`

## Current posture

- corrupted or partial run metadata does not panic run-history routes
- recovery behavior remains deterministic and auditable
- damaged-run diagnostics remain operator-visible
