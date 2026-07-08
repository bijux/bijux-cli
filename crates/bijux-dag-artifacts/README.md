# bijux-dag-artifacts

`bijux-dag-artifacts` handles artifact identity, persistence, integrity, and
lifecycle helpers for DAG runs.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate owns the
verified-artifact and persisted-evidence part of that promise.

## Release Status

- public crate on the `v0.4.0` DAG release line
- durable evidence boundary for manifests, traces, indexes, and artifact
  lineage

## What This Crate Owns

- run-manifest, node-trace, outputs-index, and storage-layout models
- artifact hashing, proof, and schema validation helpers
- filesystem-backed storage and layout helpers
- retention, promotion, and lineage policy primitives
- the retained run-directory, cache-entry, and promotion-ledger evidence shapes

Use this crate when you need to read, write, validate, or transport DAG run
artifacts without taking on runtime execution or command orchestration.

## Good Fit

- reading retained run evidence from Rust
- validating manifests, traces, indexes, and artifact hashes
- moving run bundles between machines or storage layers
- working with promotion, retention, and lineage data without embedding the
  runtime

## What It Does Not Own

- graph semantics or planner logic
- scheduler and execution policy behavior
- CLI command routing
- maintainer governance flows

## Public Rust Surface

- browse docs.rs through `bijux_dag_artifacts::stable` for the long-lived
  artifact compatibility lane
- use `bijux_dag_artifacts::prelude` for common read, write, and validation
  workflows
- use focused crate-root imports only when you already know the exact artifact
  item you need
- broad compatibility re-exports remain callable for repository-owned support
  work, but stay hidden from the primary docs.rs lane

## Source Layout

- `src/storage`: persisted run and artifact models
- `src/layout`: run-directory layout and artifact path rules
- `src/integrity`: hashes, proofs, and verification helpers
- `src/lifecycle`: retention, promotion, and lineage helpers
- `src/io`: filesystem read and write helpers

## Reach For Another Crate When

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
- [Reproducibility model](https://bijux.io/bijux-core/bijux-dag/interfaces/reference/reproducibility-model/)
- [Run evidence layout](https://bijux.io/bijux-core/bijux-dag/interfaces/reference/run-evidence-layout/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
