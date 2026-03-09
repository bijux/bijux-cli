# Performance Optimization Telemetry Report

## Scope

Performance telemetry for optimization tracks benchmark latency, throughput, and profiling outcomes.

## Telemetry-backed surfaces

- scheduler latency: `docs/reports/foundation/RUNTIME_ENGINE_SCHEDULER_HOTPATH_BENCHMARK.md`
- run history latency: `docs/reports/foundation/run_history_query_latency_report.md`
- artifact inspect/hash/trace latency: `docs/reports/foundation/artifact_inspect_verify_latency_report.md`
- replay and diff latency: `docs/reports/foundation/replay_proof_latency_report.md`, `docs/reports/foundation/diff_explain_latency_report.md`
- workflow memory profile: `docs/reports/foundation/WORKFLOW_MEMORY_BENCHMARKS.md`

## Governance anchors

- contract: `docs/spec/PERFORMANCE_OPTIMIZATION_CONTRACT.md`
- suite: `configs/suites/performance_optimization_regression.json`
- corpus: `evidence/cache/performance_optimization/regression_corpus.json`
