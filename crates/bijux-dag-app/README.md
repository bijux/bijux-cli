# bijux-dag-app

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-app?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-app)
[![Rust docs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/) [![bijux-dag-app docs](https://img.shields.io/badge/docs-app-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-app` is the application layer behind the `bijux-dag` command
surface. It translates command intent into calls across the DAG crates, applies
release-boundary routing, and shapes the typed responses that the CLI renders.

`bijux-dag` v0.4.1 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate keeps
that product promise coherent at the command and response boundary.

## Release Status

- public crate on the `v0.4.1` DAG release line
- owns the command application layer, not the thin binary wrapper
- contains repository-owned experimental and opt-in routes, but those routes
  are not automatically part of the stable operator contract

## Good Fit

- embedding `bijux-dag` command behavior in another Rust surface
- reusing Bijux command orchestration without re-implementing route policy
- shaping the same machine and human response models used by the CLI
- inspecting how stable, experimental, simulated, and internal lanes are gated

## What This Crate Provides

- command orchestration and request validation at the app boundary
- typed response models and render helpers
- user-facing flows for inspection, replay, cache work, graph inspection,
  migration, and diagnostics
- run summaries and failure explanations that surface container engine
  availability, failed node classes, and retained trace locations
- node inspection that surfaces retained terminal log paths, byte sizes,
  bounded tail excerpts, and process exit codes when the runtime recorded them
- branch-facing command flows that surface selected decisions, skipped lanes,
  join trigger outcomes, and replay proof summaries
- route gating between stable, experimental, simulated, and internal surfaces
- lane-scoped command discovery for stable, experimental, simulated, and
  maintainer route inventories

## What It Does Not Own

- graph semantics or canonical validation rules
- scheduler and runtime execution internals
- artifact storage implementations
- maintainer-only governance workflows

## Public Rust Surface

- browse docs.rs through `bijux_dag_app::stable` for the long-lived command
  application lane
- use `bijux_dag_app::prelude` for command embedding helpers
- use focused crate-root imports only when you already know the exact app item
  you need
- broad compatibility re-exports remain callable for repository-owned support
  work, but stay hidden from the primary docs.rs lane

## Source Layout

- `src/commands`: Clap model, release-boundary help shaping, and command policy
- `src/routes`: command-to-service routing and public-versus-hidden route gates
- `src/inspect`: run inspection, failure explanation, and comparison views
- `src/replay`: replay planning, verification, and focused diff surfaces
- `src/graph`: graph-level validation and inspection helpers
- `src/cache`, `src/read`, `src/write`, `src/explain`, `src/format`: support
  modules for app-layer workflows

## Reach For Another Crate When

- you need deterministic graph truth or planner primitives:
  `bijux-dag-core`
- you need execution policy, replay reuse rules, or runtime diagnostics:
  `bijux-dag-runtime`
- you need persisted evidence models or integrity helpers:
  `bijux-dag-artifacts`
- you only need the executable boundary:
  `bijux-dag-cli`

## Representative Workflows

For the repository-backed example that shows how the app surface reports a real
cache verification and diagnostic sequence, including changed-input cache
misses and corruption-based reuse refusal, use
[Cache Behavior Workflow](https://bijux.io/bijux-core/bijux-dag/operations/cache-behavior-workflow/).

For the canonical explanation of which retained fingerprints and bundle modes
those commands are actually reporting, use
[Reproducibility Model](https://bijux.io/bijux-core/bijux-dag/interfaces/reproducibility-model/).

For the repository-backed example that shows how the app surface reports a real
container run, retained outputs, and a missing-engine infrastructure failure,
use
[Container Packaging Workflow](https://bijux.io/bijux-core/bijux-dag/operations/container-packaging-workflow/).

For the repository-backed example that shows how the app surface reports a real
branch decision, a skipped lane, and replay stability at the publication
boundary, use
[Branching Bulletin Workflow](https://bijux.io/bijux-core/bijux-dag/operations/branching-bulletin-workflow/).

For the repository-backed example that shows how the app surface separates root
failure from propagated skips, replays only the failed approval boundary, and
verifies the repaired run strictly, use
[Compliance-Gated Bulletin Workflow](https://bijux.io/bijux-core/bijux-dag/operations/compliance-gated-bulletin-workflow/).

Repository-owned schedule and backfill application flows are documented in the
DAG handbook, but they remain internal workflow lanes rather than part of the
default public `v0.4.x` app story.

## Internal Documentation

- [`ARCHITECTURE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/ARCHITECTURE.md): request flow, source boundaries,
  dependency direction, stable exports, and extension decisions.
- [`COMMAND_ROUTING.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/COMMAND_ROUTING.md): command authority, surface
  lanes, preconditions, paths, dispatch, and route verification.
- [`CONTRACTS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/CONTRACTS.md): owned orchestration, input, output,
  stability, dependency, and failure contracts.
- [`RESPONSES_AND_FAILURES.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/RESPONSES_AND_FAILURES.md): typed
  responses, JSON/human parity, failure classes, causality, and references.
- [`ROUTE_AUTHORING.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/ROUTE_AUTHORING.md): command ownership, lane
  policy, dispatch, preconditions, response integrity, and route review.
- [`WORKFLOWS_AND_SERVICES.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/WORKFLOWS_AND_SERVICES.md): graph, run,
  evidence, cache, replay, configuration, and service design.

## Related links

- [Crate contracts](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/docs/CONTRACTS.md)
- [Crate changelog](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-app/CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/)
