# Portability Scorecard

Generated from import/export portability tests.

## Coverage

- artifact-heavy roundtrip
- replay from imported bundle
- diff against original run
- optional/required payload behavior
- corruption rejection
- offline inspection
- cross-machine path portability
- line-ending portability
- backward-compatible fixture import

## Current status

All listed portability contract tests pass in current suite.

## Raw evidence references

- scenarios: `evidence/perf/scenarios/portability_canonical.json`
- score benchmark: `evidence/perf/scenarios/portability_success_rate.json`
- baseline fixture set: `evidence/perf/baselines/benchmark_baseline_fixtures_v1.json`
