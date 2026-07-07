# bijux-dag-artifacts

`bijux-dag-artifacts` owns artifact identity, persistence, integrity, and
lifecycle helpers for DAG runs.

## What this crate provides

- Run-manifest, node-trace, outputs-index, and storage-layout models.
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

## Source layout

- `src/storage`: persisted run and artifact models
- `src/layout`: run-directory layout and artifact path rules
- `src/integrity`: hashes, proofs, and verification helpers
- `src/lifecycle`: retention, promotion, and lineage helpers
- `src/io`: filesystem read and write helpers

## Reach for another crate when

- you need execution-time policy or replay classification:
  `bijux-dag-runtime`
- you need command orchestration or human-readable output shaping:
  `bijux-dag-app`
- you need graph parsing, validation, or planner lowering:
  `bijux-dag-core`

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
