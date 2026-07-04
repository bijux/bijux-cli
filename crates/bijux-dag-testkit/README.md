# bijux-dag-testkit

`bijux-dag-testkit` centralizes deterministic test fixtures, builders, and
assertion helpers shared across the DAG workspace.

## What this crate provides

- Canonical DAG fixture builders.
- Shared run-artifact and replay assertions.
- Reusable helpers for contract, integration, and regression suites.

Use this crate in workspace tests when you need stable DAG inputs and shared
assertions without duplicating fixtures across crates.

## Deliberate boundaries

This crate is test-only support. It does not own:

- production command routing,
- runtime state machines,
- release-governance decisions.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-testkit/)
