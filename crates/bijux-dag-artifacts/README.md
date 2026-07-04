# bijux-dag-artifacts

`bijux-dag-artifacts` owns artifact identity, persistence, integrity, and
lifecycle helpers for DAG runs.

## What this crate provides

- Run-manifest, node-trace, and outputs-index models.
- Artifact hashing, proof, and schema validation helpers.
- Filesystem-backed storage and layout helpers.
- Retention, promotion, and lineage policy primitives.

Depend on this crate when you need to read, write, validate, or transport DAG
run artifacts without taking on the runtime or command orchestration layers.

## Deliberate boundaries

This crate does not own:

- graph semantics or planner logic,
- scheduler and execution policy behavior,
- CLI command routing or maintainer governance flows.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
