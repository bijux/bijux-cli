# Performance baseline

- Baseline command: `cargo run -p bijux-dev-dag -- benchmark-baseline`
- Fixture families:
  - `benchmarks/fixtures/large_dag.json`
  - `benchmarks/fixtures/scheduler_linear_32.json`
  - `benchmarks/fixtures/scheduler_parallel_64.json`
  - `benchmarks/fixtures/scheduler_diamond_fanout.json`
- Output artifact: `artifacts/benchmarks/baseline.json`

The benchmark baseline is deterministic and intended for trend tracking across graph families, not one fixture.
