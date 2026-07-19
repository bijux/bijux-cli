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

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-artifacts docs](https://img.shields.io/badge/docs-artifacts-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-artifacts` owns run evidence material: artifact models, persistence
helpers, storage layout, integrity proofs, and lifecycle policy helpers.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
The [Replay Contract](../../spec/REPLAY_CONTRACT.md) defines the replay authority.
This package owns the verified-artifact and persisted-evidence part of that
promise.

Use this page when the question is about what a DAG run leaves behind, how that
material is identified, and how artifact integrity is verified over time.

For the exact retained filesystem map, open
[Run Evidence Layout](../interfaces/run-evidence-layout.md).

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

- open [Reproducibility Model](../interfaces/reproducibility-model.md)
  when the question is how retained artifact hashes and producer fingerprints
  participate in cache proof and replay proof
- open [Run Evidence Layout](../interfaces/run-evidence-layout.md)
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
- `crates/bijux-dag-artifacts/docs/CONTRACTS.md`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/integrity/proof.rs`
- `crates/bijux-dag-artifacts/src/lifecycle/retention.rs`

## Review Lens

- artifact identity and integrity should stay authoritative and inspectable
- persisted evidence shapes should stay stable enough for inspection and replay
- lifecycle helpers should not drift into execution-engine policy
- retention and promotion rules should remain explicit enough to audit
