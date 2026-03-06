# Performance baseline (provisional)

- Baseline command: `cargo run -p bijux-dev-dag -- benchmark-baseline`
- Fixture families:
  - `benchmarks/fixtures/large_dag.json`
  - `benchmarks/fixtures/scheduler_linear_32.json`
  - `benchmarks/fixtures/scheduler_parallel_64.json`
  - `benchmarks/fixtures/scheduler_diamond_fanout.json`
- Output artifact: `artifacts/benchmarks/baseline.json`

This baseline is provisional and intended for early trend tracking across graph families.
It is not a release guarantee until strict measured performance gates are enforced.
