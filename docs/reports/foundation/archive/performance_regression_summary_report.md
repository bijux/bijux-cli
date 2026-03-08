# Performance Regression Summary Report

## Summary

Performance regression governance is enforced through benchmark contracts, threshold assertions, and trend diagnostics.

## Key enforcement surfaces

- `crates/bijux-dev-dag/tests/benchmark_completion_contracts.rs`
- `crates/bijux-dev-dag/tests/benchmark_signal_quality_contracts.rs`
- `crates/bijux-dev-dag/tests/perf_evidence_contracts.rs`
- `docs/reports/foundation/benchmark_threshold_assertions_run_history.json`
- `docs/reports/foundation/benchmark_threshold_assertions_runtime_helpers.json`

## Release expectation

Regression signals are triaged before release when any threshold assertion report indicates drift beyond policy limits.
