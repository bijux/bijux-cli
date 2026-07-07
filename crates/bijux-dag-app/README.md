# bijux-dag-app

`bijux-dag-app` is the application layer behind the `bijux-dag` command
surface. It translates command intent into calls across the DAG crates, owns
release-boundary routing, and shapes the typed responses that the CLI renders.

## Release Status

- public crate on the `v0.4.0` DAG release line
- owns the command application layer, not the thin binary wrapper
- contains repository-owned experimental and opt-in routes, but those routes
  are not automatically part of the stable operator contract

## What This Crate Owns

- command orchestration and request validation at the app boundary
- typed response models and render helpers
- user-facing flows for inspection, replay, cache work, graph inspection,
  migration, and diagnostics
- run summaries and failure explanations that surface container engine
  availability, failed node classes, and retained trace locations
- branch-facing command flows that surface selected decisions, skipped lanes,
  join trigger outcomes, and replay proof summaries
- route gating between stable, experimental, simulated, and internal surfaces

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

## Representative Workflow

For the repository-backed example that shows how the app surface reports a real
cache verification and diagnostic sequence, including changed-input cache
misses and corruption-based reuse refusal, use
[Cache Behavior Workflow](../../docs/bijux-dag/operations/guides/cache-behavior-workflow.md).

For the repository-backed example that shows how the app surface reports a real
container run, retained outputs, and a missing-engine infrastructure failure,
use
[Container Packaging Workflow](../../docs/bijux-dag/operations/guides/container-packaging-workflow.md).

For the repository-backed example that shows how the app surface reports a real
branch decision, a skipped lane, and replay stability at the publication
boundary, use
[Branching Bulletin Workflow](../../docs/bijux-dag/operations/guides/branching-bulletin-workflow.md).

For the repository-backed example that shows how the app surface separates root
failure from propagated skips, replays only the failed approval boundary, and
verifies the repaired run strictly, use
[Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md).

For the repository-backed example that shows how the app surface reports
internal schedule preview, same-slot suppression, queue dispatch, explicit
ledger completion, and one run id carried through to the final manifest, use
[Scheduled Catalog Refresh Workflow](../../docs/bijux-dag/operations/guides/scheduled-catalog-refresh-workflow.md).

For the repository-backed example that shows how the app surface reports
backfill partition fanout, aggregate summary counts, failed-partition retry,
and explicit run handoff, use
[Historical Catalog Backfill Workflow](../../docs/bijux-dag/operations/guides/historical-catalog-backfill-workflow.md).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/)
