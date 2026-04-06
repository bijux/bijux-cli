# bijux-dag-cli

Thin binary entrypoint crate that wires CLI parsing and error mapping.
Responsibility: Thin process entrypoint that delegates to app command surfaces.

## Why this crate exists
This crate provides the executable shell and process wiring for user-facing command invocation.

## What must never enter this crate
- DAG semantic logic.
- Runtime execution state machines.
- Artifact storage internals.

See [CONTRACT.md](./CONTRACT.md).
