---
title: bijux-dag-artifacts Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# bijux-dag-artifacts

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
