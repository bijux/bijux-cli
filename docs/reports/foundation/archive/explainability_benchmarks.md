# Explainability Benchmarks

Generated: 2026-03-08

## Scope
Tracks explain-surface performance and stability for:
- `why-rerun`
- `why-cache-missed`
- `trace-artifact`
- diff/replay explain reason grouping

## Governed benchmark references
- `docs/reports/foundation/diff_explain_latency_report.md`
- `docs/reports/foundation/semantic_diff_explain_benchmarks.md`
- `docs/reports/foundation/app_inspect_explain_latency_baseline.md`
- `evidence/perf/scenarios/explainability_quality.json`

## Current status
- explain outputs are schema-governed and snapshot-governed
- no release-blocking explain latency regressions are recorded in this snapshot

## Contract links
- `docs/spec/EXPLAIN_SURFACES_CONTRACT.md`
- `configs/suites/explain_surface_stress.json`
- `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`
