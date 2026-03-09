# Failure Handling Benchmarks

Generated: 2026-03-08

## Scope
Tracks performance and signal quality for runtime failure handling and recovery paths:
- failure class decision overhead
- interruption recovery decision latency
- failure-injection workflow cost
- failure explain payload generation overhead

## Governed benchmark references
- `evidence/perf/scenarios/failure_injection_canonical.json`
- `docs/reports/foundation/flaky_noisy_benchmark_report.md`
- `docs/reports/foundation/slow_benchmark_signal_value_report.md`
- `docs/reports/foundation/RUNTIME_ENGINE_SCHEDULER_HOTPATH_BENCHMARK.md`

## Current status
- failure-injection scenario is governed and retained
- no release-blocking benchmark drift is recorded in this snapshot
- benchmark signals remain advisory unless explicitly promoted by policy

## Contract links
- `docs/spec/FAILURE_TAXONOMY_CONTRACT.md`
- `docs/RUN_RECOVERY_AND_RESILIENCE.md`
- `crates/bijux-dev-dag/tests/failure_recovery_completion_contracts.rs`
