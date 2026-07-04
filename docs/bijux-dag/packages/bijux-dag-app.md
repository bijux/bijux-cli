---
title: bijux-dag-app Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-dag-app

`bijux-dag-app` is the application orchestration layer for `bijux-dag`
surfaces. It translates command inputs into services, coordinates reads and
writes, and shapes user-facing responses.

Use this page when the issue is about command behavior or output shape rather
than graph truth or execution internals.

This crate also houses hidden simulation and maintainer routes that stay in the
repository for coverage and evidence work. Those paths are intentionally kept
outside the visible `bijux-dag --help` release contract.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| command orchestration | argument-to-service routing, workflow dispatch, output selection |
| response shaping | render flows, response models, diagnostics views, command-specific output contracts |
| app-level services | read, write, replay, inspect, graph, cache, and export/import orchestration |
| boundary | does not own kernel semantics, runtime scheduler internals, or artifact storage authority |

## Source Layout

- `crates/bijux-dag-app/src/commands`
- `crates/bijux-dag-app/src/routes`
- `crates/bijux-dag-app/src/inspect`
- `crates/bijux-dag-app/src/replay`
- `crates/bijux-dag-app/src/graph`
- `crates/bijux-dag-app/src/format`
- `crates/bijux-dag-app/src/read`
- `crates/bijux-dag-app/src/write`

## Open Next

- open the [DAG Handbook](../../index.md) for the package-wide architecture and interfaces
- open [`bijux-dag-runtime`](./bijux-dag-runtime.md) when the question crosses from response shaping into execution policy
- open [`bijux-dag-cli`](./bijux-dag-cli.md) when the concern is process wiring rather than app orchestration

## Code Anchors

- `crates/bijux-dag-app/README.md`
- `crates/bijux-dag-app/CONTRACT.md`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dag-app/src/routes/run_routes.rs`
- `crates/bijux-dag-app/src/inspect/service.rs`

## Review Lens

- command routing should stay thin enough to explain and thick enough to keep user-facing contracts coherent
- orchestration should delegate kernel and runtime work instead of re-implementing it
- hidden maintainer routes must not blur the public operator-facing contract
- output contracts should remain explicit and test-backed
