# Runtime Contract Tests

This suite verifies execution after a graph has been validated and planned.
It covers scheduling, node execution, cache decisions, replay, policy,
backends, observability, recovery, and retained lifecycle evidence. Command
wording belongs to `bijux-dag-app`; graph semantics belong to
`bijux-dag-core`.

## Coverage

- deterministic readiness, concurrency, cancellation, and terminal state
- node attempts, retries, timeouts, subprocess cleanup, and recovery
- cache identity, invalidation, corruption refusal, and replay equivalence
- artifact handoff and lifecycle evidence emitted by execution
- environment, filesystem, clock, network, and authorization policy
- local, container, batch, Kubernetes, adapter, and modeled backend boundaries
- event completeness, redaction, diagnostics, and formal runtime invariants

The presence of modeled platform tests does not make those platforms public
product surfaces. Public support is governed by the DAG release boundary and
backend capability contracts.

## Focused Runs

```bash
cargo nextest run -p bijux-dag-runtime --test scheduler_contract
cargo nextest run -p bijux-dag-runtime --test cache_contracts
cargo nextest run -p bijux-dag-runtime --test runtime_failure_contracts
cargo nextest run -p bijux-dag-runtime --test policy_cache_contract
```

Use a test binary that owns the failed behavior before widening the run.
Concurrency tests must prove eventual state with bounded coordination; sleeps
are not a substitute for a synchronization contract. Backend tests must state
whether they exercise a real adapter, a protocol boundary, or a simulation.

## Fixtures And Artifacts

Reuse governed scenarios under `evidence/dag/` when the behavior crosses
crates. Keep test-local inputs minimal and deterministic. Runtime output must
stay in test temporary directories or the repository `artifacts/` root.
