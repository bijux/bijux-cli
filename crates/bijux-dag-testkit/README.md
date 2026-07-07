# bijux-dag-testkit

`bijux-dag-testkit` centralizes deterministic test fixtures, builders, and
assertion helpers shared across the DAG workspace.

## Release Status

- repository-internal support crate
- not part of the public `v0.4.0` crates.io release boundary
- intended for workspace crates and top-level suites

## What This Crate Owns

- canonical DAG fixture builders
- shared run-artifact and replay assertions
- reusable helpers for contract, integration, regression, and fake-adapter
  suites

Use this crate in workspace tests when you need stable DAG inputs and shared
assertions without duplicating fixtures across crates.

## What It Does Not Own

- production command routing
- runtime state machines
- crates.io publication policy
- release-governance decisions

## Source Layout

- `src/workflows.rs`: reusable workflow fixtures
- `src/product_scenarios.rs`: cross-crate scenario builders
- `src/fake_adapter.rs`: deterministic adapter harness support
- `src/lib.rs`: exported fixture and assertion helpers

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-testkit/)
