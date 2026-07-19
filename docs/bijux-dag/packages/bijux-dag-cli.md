---
title: bijux-dag-cli Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# bijux-dag-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-cli docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-cli` is the thin binary entrypoint for DAG commands. It owns process
wiring, argument handoff, and exit-code mapping, while delegating DAG semantics
to the application layer.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles.
The [Replay Contract](../../spec/REPLAY_CONTRACT.md) defines the replay authority.
This package is the installed binary entrypoint for that promise.

Use this page when the issue is about executable startup, process behavior, or
binary-level integration rather than DAG semantics themselves.

The supported operator contract is the visible `bijux-dag --help` surface.
That visible root surface stays intentionally concise for `v0.4.0`. Hidden
experimental routes remain executable by explicit path and are inventoryable
through `bijux-dag commands --lane experimental`. Simulation namespaces and
maintainer namespaces require `BIJUX_DAG_ENABLE_SIMULATED=1` or
`BIJUX_DAG_ENABLE_INTERNAL=1`, plus deliberate inventory through
`bijux-dag commands --lane simulated` or `bijux-dag commands --lane internal`.
`bijux-dag-cli` does not advertise them as stable public behavior.

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
- open [Reproducibility Model](../interfaces/reproducibility-model.md) when the visible `bijux-dag` command output is really a question about retained identity, cache proof, or replay-bundle semantics
- open [Cache Behavior Workflow](../operations/cache-behavior-workflow.md) for a repository-backed cache verification and explicit-path diagnostic sequence that still runs through the published `bijux-dag` binary
- open [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md) for a repository-backed recovery path that stays entirely on the public `bijux-dag` command surface
- open the [Repository Handbook](../../bijux-core/index.md) when process behavior intersects shared release policy

## Internal Evidence Lanes

- open [Historical Catalog Backfill Workflow](../operations/historical-catalog-backfill-workflow.md) only when you intentionally need the repository-backed internal backfill lane through `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`
- open [Scheduled Catalog Refresh Workflow](../operations/scheduled-catalog-refresh-workflow.md) only when you intentionally need the repository-backed internal schedule lane through `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`

## Code Anchors

- `crates/bijux-dag-cli/README.md`
- `crates/bijux-dag-cli/docs/CONTRACTS.md`
- `crates/bijux-dag-cli/src/main.rs`

## Review Lens

- the binary should stay thin enough that DAG behavior remains owned elsewhere
- user-facing startup and exit behavior should still be explicit and testable
- repository-owned experimental routes must stay intentionally outside the default root help surface until they are promoted
- modeled-platform and maintainer namespaces must stay intentionally outside the public root help surface
- process-level concerns should not pull runtime or artifact logic into the entrypoint
