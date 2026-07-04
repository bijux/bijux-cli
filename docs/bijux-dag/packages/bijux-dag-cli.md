---
title: bijux-dag-cli Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-dag-cli

`bijux-dag-cli` is the thin binary entrypoint for DAG commands. It owns process
wiring, argument handoff, and exit-code mapping, while delegating DAG semantics
to the application layer.

Use this page when the issue is about executable startup, process behavior, or
binary-level integration rather than DAG semantics themselves.

The supported operator contract is the visible `bijux-dag --help` surface.
Hidden simulation and maintainer namespaces remain executable by explicit path,
but `bijux-dag-cli` does not advertise them as stable public behavior.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| process entrypoint | binary startup, argv handoff, and error mapping |
| runtime shell | thin executable wrapper for user-facing invocation |
| boundary | does not own graph semantics, execution policy, or artifact storage |

## Source Layout

- `crates/bijux-dag-cli/src/main.rs`

## Open Next

- open [`bijux-dag-app`](./bijux-dag-app.md) for command orchestration and user-facing response shaping
- open the [DAG Handbook](../../index.md) for the wider system map and operator guidance
- open the [Repository Handbook](../../bijux-core/index.md) when process behavior intersects shared release policy

## Code Anchors

- `crates/bijux-dag-cli/README.md`
- `crates/bijux-dag-cli/CONTRACT.md`
- `crates/bijux-dag-cli/src/main.rs`

## Review Lens

- the binary should stay thin enough that DAG behavior remains owned elsewhere
- user-facing startup and exit behavior should still be explicit and testable
- hidden maintainer namespaces must stay intentionally outside the public root help surface
- process-level concerns should not pull runtime or artifact logic into the entrypoint
