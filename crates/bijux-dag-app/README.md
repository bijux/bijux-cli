# bijux-dag-app

`bijux-dag-app` is the application layer behind the `bijux-dag` command
surface. It translates user intent into calls across the DAG core, runtime, and
artifact crates, then shapes the resulting typed responses.

## What this crate provides

- Command orchestration and request validation at the app boundary.
- Typed response models and render helpers.
- User-facing flows for inspect, replay, cache, migration, and diagnostics.

Depend on this crate when you need to embed or test the `bijux-dag` command
application logic without taking on the thin binary wrapper.

## Deliberate boundaries

This crate does not own:

- graph semantics or canonical validation rules,
- scheduler and runtime execution internals,
- artifact storage implementations or maintainer-only governance workflows.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/)
