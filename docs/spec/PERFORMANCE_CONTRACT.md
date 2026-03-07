# Performance Contract

## Scope
Defines benchmark classes, canonical system scenarios, evidence requirements, and claim discipline.

## Allowed claims
- Performance language in documentation must reference:
  - `benchmarks/` scenarios, and
  - `artifacts/benchmarks/` or `benchmarks/baselines/` evidence.
- Claims about speed, efficiency, or low overhead without evidence links are non-compliant.

## Benchmark classes
- microbenchmark: isolated crate-level measurement.
- system benchmark: end-to-end DAG command execution with run artifacts.

## Canonical system scenarios
- tiny: `benchmarks/scenarios/tiny_canonical.json`
- medium: `benchmarks/scenarios/medium_canonical.json`
- wide: `benchmarks/scenarios/wide_canonical.json`
- deep: `benchmarks/scenarios/deep_canonical.json`
- cache-heavy: `benchmarks/scenarios/cache_heavy_canonical.json`
- replay: `benchmarks/scenarios/replay_canonical.json`

## Battle scenarios
- scheduler overhead on many tiny tasks: `benchmarks/scenarios/many_small_nodes_scheduler_overhead.json`
- artifact write amplification: `benchmarks/scenarios/manifest_trace_write_amplification.json`
- replay verification cost: `benchmarks/scenarios/replay_verification_cost.json`

## Structured output
- Baseline and measured benchmark outputs must satisfy:
  - `benchmarks/baselines/benchmark_report.schema.json`
- Required fields include benchmark format, machine metadata, commit SHA, and scenario results.

## Regression thresholds
- Trusted threshold policy is defined in `benchmarks/baselines/regression_thresholds.json`.
- `bijux-dev-dag benchmark-compare` is the baseline comparison command.

## Related tests
- `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
- `crates/bijux-dag-runtime/tests/performance_capacity_contracts.rs`

## Versioning and change policy
- Changes to scenario semantics require updating scenario owner/version metadata.
- Schema changes require compatible reader strategy and migration notes.
