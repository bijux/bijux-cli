# bijux-dag-app

`bijux-dag-app` is the application layer behind the `bijux-dag` command
surface. It translates user intent into calls across the DAG core, runtime, and
artifact crates, then shapes the resulting typed responses.

## What this crate provides

- Command orchestration and request validation at the app boundary.
- Typed response models and render helpers.
- User-facing flows for inspect, replay, cache, graph inspection, migration,
  and diagnostics.

Depend on this crate when you need to embed or test the `bijux-dag` command
application logic without taking on the thin binary wrapper.

The visible `bijux-dag --help` surface is the public operator contract. Hidden
simulation and maintainer namespaces are still routed here for repository-owned
coverage, but they are not release-grade public API.

## Deliberate boundaries

This crate does not own:

- graph semantics or canonical validation rules,
- scheduler and runtime execution internals,
- artifact storage implementations or maintainer-only governance workflows.

## Source layout

- `src/commands`: Clap model, release-boundary help shaping, and command policy
- `src/routes`: command-to-service routing and public-versus-hidden route gates
- `src/inspect`: run inspection, failure explanation, and comparison views
- `src/replay`: replay planning, verification, and focused diff surfaces
- `src/graph`: graph-level validation and inspection helpers
- `src/cache`, `src/read`, `src/write`, `src/explain`, `src/format`: support
  modules for app-layer workflows

## Reach for another crate when

- you need deterministic graph truth or planner primitives:
  `bijux-dag-core`
- you need execution policy, replay reuse rules, or runtime diagnostics:
  `bijux-dag-runtime`
- you need persisted evidence models or integrity helpers:
  `bijux-dag-artifacts`
- you only need the executable boundary:
  `bijux-dag-cli`

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/)
