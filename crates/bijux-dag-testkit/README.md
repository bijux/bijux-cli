# bijux-dag-testkit

`bijux-dag-testkit` centralizes deterministic test fixtures, builders, and
assertion helpers shared across the DAG workspace.

## Release Status

- repository-internal support crate
- not part of the public `v0.4.0` crates.io release boundary
- intended for workspace crates and top-level suites

## Good Fit

- building DAG integration or contract suites without duplicating fixtures
- reusing canonical run-artifact and replay assertions across crates
- exercising fake-adapter flows with the same harnesses used by repository
  tests
- keeping repository scenarios aligned when graph or runtime behavior changes

## What It Provides

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

## Internal Documentation

- [`ARCHITECTURE.md`](docs/ARCHITECTURE.md): private test-support boundaries,
  dependency direction, determinism, and extension decisions.
- [`ASSERTIONS_AND_HARNESSES.md`](docs/ASSERTIONS_AND_HARNESSES.md): semantic
  normalization, trace laws, command isolation, and corruption builders.
- [`CONTRACTS.md`](docs/CONTRACTS.md): fixture, fake, assertion, dependency,
  publication, and change contracts.
- [`FAKES_AND_SCENARIOS.md`](docs/FAKES_AND_SCENARIOS.md): adapter fakes,
  deterministic outcomes, product scenario reports, and fault design.
- [`FIXTURES_AND_BUILDERS.md`](docs/FIXTURES_AND_BUILDERS.md): graph builders,
  repository assets, synthetic/evidence distinction, and snapshots.
- [`HERMETIC_TESTS.md`](docs/HERMETIC_TESTS.md): per-test resource ownership,
  process and environment isolation, golden updates, and narrow serialization.

## Related links

- [Crate contracts](docs/CONTRACTS.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-testkit/)
