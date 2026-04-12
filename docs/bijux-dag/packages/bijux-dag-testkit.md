---
title: bijux-dag-testkit Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-dag-testkit

`bijux-dag-testkit` centralizes deterministic test fixtures, builders, and
assertion helpers shared across DAG crates and top-level suites.

Use this page when the work is about DAG test support, reproducible fixtures,
or shared assertions rather than production runtime behavior.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| shared fixtures | reusable graph and runtime test material |
| helpers | builders and assertions used across DAG suites |
| boundary | does not own production command routing, runtime policy, or release governance |

## Source Layout

- `crates/bijux-dag-testkit/src/lib.rs`

## Open Next

- open the [DAG Handbook](../../index.md) when the testing question belongs to the wider DAG stack
- open [`bijux-dag-core`](./bijux-dag-core.md) or [`bijux-dag-runtime`](./bijux-dag-runtime.md) when a fixture needs to map back to owned behavior
- open the [Maintainer Handbook](../../bijux-dev/index.md) when the concern is repository-level verification policy

## Code Anchors

- `crates/bijux-dag-testkit/README.md`
- `crates/bijux-dag-testkit/CONTRACT.md`
- `crates/bijux-dag-testkit/src/lib.rs`

## Review Lens

- shared test helpers should improve determinism without hiding the behavior under test
- production semantics should stay in product crates, not in fixture glue
- repository-level test policy should still live in maintainer docs rather than this package page
