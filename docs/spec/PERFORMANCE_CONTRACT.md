# Performance Contract

## Scope
Defines benchmark classes, canonical system scenarios, evidence requirements, and claim discipline.

## Allowed claims
- Performance language in documentation must reference:
  - `evidence/perf/scenarios/` canonical workloads, and
  - `artifacts/benchmarks/` or `evidence/perf/baselines/` evidence.
- Claims about speed, efficiency, or low overhead without evidence links are non-compliant.

## Benchmark classes
- microbenchmark: isolated crate-level measurement.
- system benchmark: end-to-end DAG command execution with run artifacts.

## Canonical system scenarios
- tiny: `evidence/perf/scenarios/tiny_canonical.json`
- medium: `evidence/perf/scenarios/medium_canonical.json`
- wide: `evidence/perf/scenarios/wide_canonical.json`
- deep: `evidence/perf/scenarios/deep_canonical.json`
- cache-heavy: `evidence/perf/scenarios/cache_heavy_canonical.json`
- replay: `evidence/perf/scenarios/replay_canonical.json`

## Battle scenarios
- scheduler overhead on many tiny tasks: `evidence/perf/scenarios/many_small_nodes_scheduler_overhead.json`
- artifact write amplification: `evidence/perf/scenarios/manifest_trace_write_amplification.json`
- replay verification cost: `evidence/perf/scenarios/replay_verification_cost.json`

## Structured output
- Baseline and measured benchmark outputs must satisfy:
  - `configs/schema/benchmarks/benchmark_report.schema.json`
- Required fields include benchmark format, machine metadata, commit SHA, and scenario results.

## Regression thresholds
- Trusted threshold policy is defined in `evidence/perf/baselines/regression_thresholds.json`.
- `bijux-dev-dag benchmark-compare` is the baseline comparison command.

## Related tests
- `crates/bijux-dev-dag/tests/benchmark_scenario_contract.rs`
- `crates/bijux-dag-runtime/tests/performance_capacity_contracts.rs`

## Versioning and change policy
- Changes to scenario semantics require updating scenario owner/version metadata.
- Schema changes require compatible reader strategy and migration notes.
