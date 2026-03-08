# Memory budget evidence

- Memory evidence is captured from benchmark and runtime observability artifacts.
- Output artifacts live under `artifacts/benchmarks/` and run-level observability files.
- Runtime emits materialization memory sampling in run artifacts:
  - `observability.metrics.json` with `before_materialization_bytes` and `after_materialization_bytes`.

Memory budget compliance is release-relevant only when tied to benchmark workload metadata
and measured environment context.

Resource budget checks are available in warning and gate modes:

- warning: `cargo run -p bijux-dev-dag -- resource-budget-check --report artifacts/benchmarks/baseline.json`
- gate: `cargo run -p bijux-dev-dag -- resource-budget-check --report artifacts/benchmarks/baseline.json --gate`
