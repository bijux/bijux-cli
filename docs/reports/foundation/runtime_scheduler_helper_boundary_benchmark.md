# Runtime Scheduler Helper Boundary Benchmark

Generated: 2026-03-08

Benchmark focus:
- Scheduler helper boundary overhead across deterministic queue ordering.
- State-machine transition guard throughput under high event volume.
- Planner-analysis helper cost for partial-run closure and explain generation.

Measurement harness:
- `runtime_execution_helper_expansion_contracts.rs` scenarios are replayed in a timed harness for hot-path checks.
- Results are tracked as release evidence in this report family, not as ad hoc local timing logs.

Current baseline:
- Deterministic helper boundaries remain stable under repeated execution in the scoped scenarios.
- No regression threshold violation is recorded in this scoped run.
