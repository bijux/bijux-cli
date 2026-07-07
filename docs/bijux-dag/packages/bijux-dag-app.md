---
title: bijux-dag-app Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# bijux-dag-app

`bijux-dag-app` is the application layer behind `bijux-dag`. It translates
command intent into service calls across the DAG crates, coordinates reads and
writes, and shapes the typed responses that the CLI renders.

Use this page when the issue is about command behavior or output shape rather
than graph truth or execution internals.

The intended Rust import lanes are the crate root, `stable`, and `prelude`.
Hidden compatibility helpers remain repository-owned, and the `experimental`
lane is opt-in behind `experimental-public-api`.

This crate also houses repository-owned experimental operator routes plus
modeled-platform and maintainer routes that stay in the repository for
coverage and evidence work. Experimental routes stay on explicit paths, while
simulated and maintainer lanes require `BIJUX_DAG_ENABLE_SIMULATED=1` or
`BIJUX_DAG_ENABLE_INTERNAL=1`. Those paths are intentionally kept outside the
visible `bijux-dag --help` release contract.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| command orchestration | argument-to-service routing, workflow dispatch, output selection, and public-versus-hidden route guardrails |
| response shaping | render flows, response models, diagnostics views, command-specific output contracts |
| app-level services | read, write, replay, inspect, graph, cache, migration, and export/import orchestration |
| container-facing operator surface | run summaries, failure reasons, and retained response shapes for container-backed nodes |
| branch-facing operator surface | retained branch decisions, skipped-lane explanations, join-trigger summaries, and replay proof output |
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

- open the [DAG Handbook](../index.md) for the package-wide architecture and interfaces
- open [`bijux-dag-runtime`](./bijux-dag-runtime.md) when the question crosses from response shaping into execution policy
- open [`bijux-dag-cli`](./bijux-dag-cli.md) when the concern is process wiring rather than app orchestration
- open [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md) for a repository-backed example of the app surface reporting a real container run and a missing-engine failure
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md) for a repository-backed example of the app surface reporting a real branch decision and replay-stable publication path

## Code Anchors

- `crates/bijux-dag-app/README.md`
- `crates/bijux-dag-app/CONTRACT.md`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dag-app/src/commands/mod.rs`
- `crates/bijux-dag-app/src/routes/run_routes.rs`
- `crates/bijux-dag-app/src/inspect/service.rs`

## Review Lens

- command routing should stay thin enough to explain and thick enough to keep user-facing contracts coherent
- orchestration should delegate kernel and runtime work instead of re-implementing it
- repository-owned experimental routes must not quietly expand the visible operator contract
- modeled-platform and maintainer routes must not blur the public operator-facing contract
- output contracts should remain explicit and test-backed
