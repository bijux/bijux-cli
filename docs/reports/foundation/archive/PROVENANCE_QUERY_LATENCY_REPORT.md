# Provenance Query Latency Report

Generated: 2026-03-08

## Scope
This report tracks latency for provenance and lineage query paths:
- artifact inspect provenance and lineage traversal
- trace-artifact payload generation
- lineage dependency and dependent lookups on large snapshots

## Governed benchmark surfaces
- `docs/reports/foundation/artifact_inspect_hash_trace_benchmarks.md`
- `evidence/perf/scenarios/artifact_lineage_completeness.json`
- `configs/suites/provenance_traceability_stress.json`

## Current status
- provenance query pathways are covered by contract tests and stress-oriented fixtures
- no release-blocking latency regressions are recorded in this snapshot

## Contract links
- `docs/spec/PROVENANCE_MODEL_CONTRACT.md`
- `docs/spec/TRACE_CONTRACT.md`
- `crates/bijux-dev-dag/tests/provenance_traceability_completion_contracts.rs`
