# bijux-dag-runtime

Runtime planning, execution, adapter invocation boundaries, policy checks, and trace emission.
Responsibility: Execution engine, scheduler behavior, policy enforcement, replay semantics, and runtime diagnostics.

## Why this crate exists
This crate owns execution-time behavior for DAG runs, including planning, scheduling, and runtime policy application.

## What must never enter this crate
- Authoritative DAG schema ownership.
- CLI command parsing and presentation routing.
- Governance release report orchestration.

See [CONTRACT.md](./CONTRACT.md).
