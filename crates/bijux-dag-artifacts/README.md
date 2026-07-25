# bijux-dag-artifacts

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-artifacts?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-artifacts)
[![Rust docs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/) [![bijux-dag-artifacts docs](https://img.shields.io/badge/docs-artifacts-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-artifacts` handles artifact identity, persistence, integrity, and
lifecycle helpers for DAG runs.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate provides
the verified-artifact and persisted-evidence layer of that promise.

## Release Status

- public crate on the `v0.4.0` DAG release line
- durable evidence boundary for manifests, traces, indexes, and artifact
  lineage

## What It Provides

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

## Internal Documentation

- [`ARCHITECTURE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/ARCHITECTURE.md): retained-evidence boundaries,
  evidence flow, compatibility surfaces, and extension decisions.
- [`CONTRACTS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/CONTRACTS.md): owned models, effects, dependency
  direction, invariants, and stability.
- [`INTEGRITY_AND_LINEAGE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/INTEGRITY_AND_LINEAGE.md): artifact
  identity, hashing, proofs, corruption, lineage, cache, and replay evidence.
- [`RUN_DIRECTORY.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/RUN_DIRECTORY.md): staging and final layout,
  atomic records, output indexing, resume, and reader rules.
- [`SCHEMA_EVOLUTION.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/SCHEMA_EVOLUTION.md): schema families, reader
  and writer compatibility, migration refusal, and integrity-preserving change
  rules.
- [`STORAGE_AND_LIFECYCLE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/STORAGE_AND_LIFECYCLE.md): store
  capabilities, filesystem safety, promotion, retention, import, and archive.

## Related links

- [Crate contracts](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/docs/CONTRACTS.md)
- [Crate changelog](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-artifacts/CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Reproducibility model](https://bijux.io/bijux-core/bijux-dag/interfaces/reproducibility-model/)
- [Run evidence layout](https://bijux.io/bijux-core/bijux-dag/interfaces/run-evidence-layout/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
