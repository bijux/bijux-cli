# bijux-dag-runtime

Runtime planning, execution, adapter invocation boundaries, policy checks, and trace emission.
Responsibility: Execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.

## Why this crate exists
This crate owns execution-time behavior for DAG runs, including planning, scheduling, and runtime policy application.

## What must never enter this crate
- Authoritative DAG schema ownership.
- CLI command parsing and presentation routing.
- Governance release report orchestration.

## Public surface shape
- Stable runtime execution APIs are exported from the crate root.
- Modeled platform, distributed-future, and product-story APIs are quarantined under `bijux_dag_runtime::simulated_platform`.

See [CONTRACT.md](./CONTRACT.md).
