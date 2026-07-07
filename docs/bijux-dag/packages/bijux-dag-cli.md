---
title: bijux-dag-cli Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# bijux-dag-cli

`bijux-dag-cli` is the thin binary entrypoint for DAG commands. It owns process
wiring, argument handoff, and exit-code mapping, while delegating DAG semantics
to the application layer.

Use this page when the issue is about executable startup, process behavior, or
binary-level integration rather than DAG semantics themselves.

The supported operator contract is the visible `bijux-dag --help` surface.
That visible root surface stays intentionally concise for `v0.4.0`. Hidden
experimental routes remain executable by explicit path. Simulation namespaces
and maintainer namespaces require `BIJUX_DAG_ENABLE_SIMULATED=1` or
`BIJUX_DAG_ENABLE_INTERNAL=1`, and `bijux-dag-cli` does not advertise them as
stable public behavior.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| process entrypoint | binary startup, argv handoff, and error mapping |
| runtime shell | thin executable wrapper for user-facing invocation and shell completions wiring |
| boundary | does not own graph semantics, execution policy, or artifact storage |

## Source Layout

- `crates/bijux-dag-cli/src/main.rs`

## Open Next

- open [`bijux-dag-app`](./bijux-dag-app.md) for command orchestration and user-facing response shaping
- open the [DAG Handbook](../index.md) for the wider system map and operator guidance
- open [Compliance-Gated Bulletin Workflow](../operations/guides/compliance-gated-bulletin-workflow.md) for a repository-backed recovery path that stays entirely on the public `bijux-dag` command surface
- open [Historical Catalog Backfill Workflow](../operations/guides/historical-catalog-backfill-workflow.md) for a repository-backed internal backfill path that stays callable through `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`
- open [Scheduled Catalog Refresh Workflow](../operations/guides/scheduled-catalog-refresh-workflow.md) for a repository-backed internal schedule path that stays callable through `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`
- open the [Repository Handbook](../../bijux-core/index.md) when process behavior intersects shared release policy

## Code Anchors

- `crates/bijux-dag-cli/README.md`
- `crates/bijux-dag-cli/CONTRACT.md`
- `crates/bijux-dag-cli/src/main.rs`

## Review Lens

- the binary should stay thin enough that DAG behavior remains owned elsewhere
- user-facing startup and exit behavior should still be explicit and testable
- repository-owned experimental routes must stay intentionally outside the default root help surface until they are promoted
- modeled-platform and maintainer namespaces must stay intentionally outside the public root help surface
- process-level concerns should not pull runtime or artifact logic into the entrypoint
