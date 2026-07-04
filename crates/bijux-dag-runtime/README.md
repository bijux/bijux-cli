# bijux-dag-runtime

`bijux-dag-runtime` is the execution engine for `bijux-dag`. It owns runtime
planning, scheduling, adapter invocation boundaries, policy checks, replay
classification, cache behavior, and trace emission.

## What this crate provides

- Execution planning and node orchestration.
- Policy evaluation and runtime diagnostics.
- Replay, diff, cache, and artifact integration behavior.
- Adapter boundaries for local and external execution backends.

Choose this crate when you need to execute validated DAG graphs or integrate
with Bijux runtime policies from Rust.

## Deliberate boundaries

This crate does not own:

- authoritative graph schema and validation rules,
- top-level command parsing or output presentation,
- release-governance and maintainer report composition.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/)
