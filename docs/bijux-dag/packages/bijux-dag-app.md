---
title: bijux-dag-app Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# bijux-dag-app

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-app?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-app)
[![Rust docs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-app docs](https://img.shields.io/badge/docs-app-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-app` is the application layer behind `bijux-dag`. It translates
command intent into service calls across the DAG crates, coordinates reads and
writes, and shapes the typed responses that the CLI renders.

At the product level, `bijux-dag` v0.4.0 is a local-first DAG runtime for
reproducible workflows with explicit graph contracts, deterministic execution
records, verified artifacts, cache explanation, and replayable run bundles (see [Replay Contract](../../spec/REPLAY_CONTRACT.md)).
This package keeps that promise coherent at the command and response boundary.

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
- open [Reproducibility Model](../interfaces/reference/reproducibility-model.md) when the question is what the app is reporting about fingerprints, cache proofs, or replay-bundle fidelity
- open [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md) for a repository-backed example of the app surface reporting a real container run and a missing-engine failure
- open [Cache Behavior Workflow](../operations/guides/cache-behavior-workflow.md) for a repository-backed example of the app surface reporting changed-input cache misses and corruption-based reuse refusal through explicit diagnostics
- open [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md) for a repository-backed example of the app surface reporting a real branch decision and replay-stable publication path
- open [Compliance-Gated Bulletin Workflow](../operations/guides/compliance-gated-bulletin-workflow.md) for a repository-backed example of the app surface reporting retry evidence, causal failure attribution, and a repaired replay boundary
- open [Historical Catalog Backfill Workflow](../operations/guides/historical-catalog-backfill-workflow.md) for a repository-backed example of the app surface reporting backfill fanout, aggregate summary counts, and failed-partition retry state
- open [Scheduled Catalog Refresh Workflow](../operations/guides/scheduled-catalog-refresh-workflow.md) for a repository-backed example of the app surface reporting internal schedule preview, same-slot suppression, queue dispatch, and run-manifest continuity

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
