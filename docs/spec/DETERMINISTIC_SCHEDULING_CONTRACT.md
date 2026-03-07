# Deterministic scheduling contract

For deterministic workloads, scheduling outcomes must be invariant across worker parallelism values.

Contract requirements:
- `jobs=1` and `jobs>1` produce equivalent manifests and outputs for deterministic DAGs.
- Ready-node tie breaking is stable for equal priority nodes.
- Failure propagation decisions are deterministic from graph state and selection state.
- Retry backoff metadata is persisted and replay-explainable.
