---
title: bijux-dag-artifacts Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

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

`bijux-dag-artifacts` owns run evidence material: artifact models, persistence
helpers, storage layout, integrity proofs, and lifecycle policy helpers.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
This package owns the verified-artifact and persisted-evidence part of that
promise.

Use this page when the question is about what a DAG run leaves behind, how that
material is identified, and how artifact integrity is verified over time.

For the exact retained filesystem map, open
[Run Evidence Layout](../interfaces/reference/run-evidence-layout.md).

The intended Rust import lanes are the crate root, `stable`, and `prelude`.
Hidden compatibility modules remain available for repository-owned coverage, and
the `experimental` lane is opt-in behind `experimental-public-api`.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| artifact identity | storage models, paths, platform layout, manifests, output indexes, and lineage material |
| integrity | hashes, proofs, schema material, and verification helpers |
| lifecycle | promotion, retention, and persistence service helpers |
| boundary | does not own CLI routing or runtime scheduler behavior |

## Source Layout

- `crates/bijux-dag-artifacts/src/storage`
- `crates/bijux-dag-artifacts/src/layout`
- `crates/bijux-dag-artifacts/src/integrity`
- `crates/bijux-dag-artifacts/src/lifecycle`
- `crates/bijux-dag-artifacts/src/io`

## Open Next

- open [Reproducibility Model](../interfaces/reference/reproducibility-model.md)
  when the question is how retained artifact hashes and producer fingerprints
  participate in cache proof and replay proof
- open [Run Evidence Layout](../interfaces/reference/run-evidence-layout.md)
  when the question is where manifests, traces, indexes, cache entries, or
  promotion records live on disk
- open [Artifact Contracts](../interfaces/artifact-contracts.md) when the
  question is which evidence surfaces are compatibility-bearing rather than only
  where they are stored
- open [`bijux-dag-runtime`](./bijux-dag-runtime.md) when artifact work is tied to execution and replay policy
- open the [DAG Handbook](../index.md) for the full DAG ownership map
- open the [Repository Handbook](../../bijux-core/index.md) when artifact contracts affect shared governance

## Code Anchors

- `crates/bijux-dag-artifacts/README.md`
- `crates/bijux-dag-artifacts/CONTRACT.md`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/integrity/proof.rs`
- `crates/bijux-dag-artifacts/src/lifecycle/retention.rs`

## Review Lens

- artifact identity and integrity should stay authoritative and inspectable
- persisted evidence shapes should stay stable enough for inspection and replay
- lifecycle helpers should not drift into execution-engine policy
- retention and promotion rules should remain explicit enough to audit
