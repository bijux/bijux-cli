---
title: bijux-dag-artifacts Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-dag-artifacts

`bijux-dag-artifacts` owns run artifact models, persistence helpers, integrity
proofs, storage hardening, and lifecycle policy helpers.

Use this page when the question is about what a DAG run leaves behind, how that
material is identified, and how artifact integrity is verified over time.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| artifact identity | storage models, paths, platform layout, and lineage material |
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

- open [`bijux-dag-runtime`](./bijux-dag-runtime.md) when artifact work is tied to execution and replay policy
- open the [DAG Handbook](../../index.md) for the full DAG ownership map
- open the [Repository Handbook](../../bijux-core/index.md) when artifact contracts affect shared governance

## Code Anchors

- `crates/bijux-dag-artifacts/README.md`
- `crates/bijux-dag-artifacts/CONTRACT.md`
- `crates/bijux-dag-artifacts/src/lib.rs`
- `crates/bijux-dag-artifacts/src/integrity/proof.rs`
- `crates/bijux-dag-artifacts/src/lifecycle/retention.rs`

## Review Lens

- artifact identity and integrity should stay authoritative and inspectable
- lifecycle helpers should not drift into execution-engine policy
- retention and promotion rules should remain explicit enough to audit
