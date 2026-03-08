# Performance Evidence Report

## Contract authority
- `docs/spec/PERFORMANCE_CONTRACT.md`

## Structured schema
- `configs/schema/benchmarks/benchmark_report.schema.json`

## Canonical scenarios
- `evidence/perf/scenarios/tiny_canonical.json`
- `evidence/perf/scenarios/medium_canonical.json`
- `evidence/perf/scenarios/wide_canonical.json`
- `evidence/perf/scenarios/deep_canonical.json`
- `evidence/perf/scenarios/cache_heavy_canonical.json`
- `evidence/perf/scenarios/replay_canonical.json`

## Battle scenarios
- `evidence/perf/scenarios/many_small_nodes_scheduler_overhead.json`
- `evidence/perf/scenarios/manifest_trace_write_amplification.json`
- `evidence/perf/scenarios/replay_verification_cost.json`

## Regression policy
- `evidence/perf/baselines/regression_thresholds.json`
- compare command: `cargo run -p bijux-dev-dag -- benchmark-compare --current <report> --baseline <report>`

## Governance checks
- suite id: `performance-claims`
- suite id: `performance-evidence`
- scenario ownership test: `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
