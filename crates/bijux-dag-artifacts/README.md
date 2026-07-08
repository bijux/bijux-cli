# bijux-dag-artifacts

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-artifacts?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-artifacts)
[![Rust docs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-artifacts docs](https://img.shields.io/badge/docs-artifacts-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
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

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Reproducibility model](https://bijux.io/bijux-core/bijux-dag/interfaces/reference/reproducibility-model/)
- [Run evidence layout](https://bijux.io/bijux-core/bijux-dag/interfaces/reference/run-evidence-layout/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
