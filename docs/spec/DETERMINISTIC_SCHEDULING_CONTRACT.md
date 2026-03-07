# Deterministic scheduling contract

For deterministic workloads, scheduling outcomes must be invariant across worker parallelism values.

## Scope
Defines deterministic scheduling behavior for planning, dispatch ordering, failure propagation, and retry accounting.

Contract requirements:
- `jobs=1` and `jobs>1` produce equivalent manifests and outputs for deterministic DAGs.
- Ready-node tie breaking is stable for equal priority nodes.
- Failure propagation decisions are deterministic from graph state and selection state.
- Retry backoff metadata is persisted and replay-explainable.

## Related tests
- `crates/bijux-dag-runtime/tests/scheduler_determinism.rs`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`

## Versioning and change policy
Any scheduling semantic change must update deterministic fixtures and the scheduler contract tests before merge.
