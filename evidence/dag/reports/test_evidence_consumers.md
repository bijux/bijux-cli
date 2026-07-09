# Test Evidence Consumer Mapping

## Purpose

This report lists the primary test surfaces and the evidence assets they consume.

## Mapping

- `crates/bijux-dag-core/tests/examples_contract.rs`
  - `evidence/authoring/examples/*.dag.json`
- `crates/bijux-dag-app/tests/comparison_harness_contract.rs`
  - `evidence/compare/scenarios/*.json`
  - `evidence/compare/baselines/bijux_v1.json`
- `crates/bijux-dag-app/tests/replay_contract.rs`
  - `evidence/cache/replay/*.json`
- `crates/bijux-dag-runtime/tests/infrastructure_fixture_contract.rs`
  - `evidence/perf/fixtures/infrastructure/backend_conformance_expectations.json`
- `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
  - `evidence/perf/scenarios/*.json`
  - `evidence/perf/baselines/*.json`

## Policy

Consumers must reference evidence-owned assets. Consumers must not reference legacy scenario roots.
