# Performance Evidence Report

## Contract authority
- `docs/spec/PERFORMANCE_CONTRACT.md`

## Structured schema
- `benchmarks/baselines/benchmark_report.schema.json`

## Canonical scenarios
- `benchmarks/scenarios/tiny_canonical.json`
- `benchmarks/scenarios/medium_canonical.json`
- `benchmarks/scenarios/wide_canonical.json`
- `benchmarks/scenarios/deep_canonical.json`
- `benchmarks/scenarios/cache_heavy_canonical.json`
- `benchmarks/scenarios/replay_canonical.json`

## Battle scenarios
- `benchmarks/scenarios/many_small_nodes_scheduler_overhead.json`
- `benchmarks/scenarios/manifest_trace_write_amplification.json`
- `benchmarks/scenarios/replay_verification_cost.json`

## Regression policy
- `benchmarks/baselines/regression_thresholds.json`
- compare command: `cargo run -p bijux-dev-dag -- benchmark-compare --current <report> --baseline <report>`

## Governance checks
- suite id: `performance-claims`
- suite id: `performance-evidence`
- scenario ownership test: `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
