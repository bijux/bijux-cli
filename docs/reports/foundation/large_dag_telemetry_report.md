# Large DAG Telemetry Report

Generated telemetry summary for large DAG workloads.

## Tracked signals

- planner queue depth for large DAG submissions
- scheduler ready-set width under fan-out/fan-in pressure
- runtime event throughput under high node counts
- replay planning counters for imported and native runs
- diff comparison counters for large runs
- provenance traversal latency buckets

## Source of truth

- `evidence/cache/scalability/regression_corpus.json`
- `configs/suites/large_dag_scalability_regression.json`
