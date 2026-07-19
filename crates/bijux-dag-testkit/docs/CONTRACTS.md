# `bijux-dag-testkit` Contracts

`bijux-dag-testkit` is private repository test support. It centralizes
deterministic graph fixtures, retained-evidence loaders, fake adapters, command
harnesses, and assertions that multiple DAG packages need.

## Owned Surface

The crate owns:

- canonical graph builders for common topology and failure shapes;
- checked fixture resolution from repository-owned data;
- typed and JSON fixture loaders;
- evidence registry lookup helpers;
- fake adapter scenarios and deterministic adapter outcomes;
- normalized manifest, trace, and event assertions;
- temporary-repository command harnesses;
- explicit corrupted run-directory builders.

It does not own product behavior, public fixtures for external consumers,
runtime algorithms, or release evidence conclusions.

## Dependency And Publication Boundary

The testkit depends on core and artifact packages because its fixtures use
their public models. Runtime, app, and maintainer packages may use it only as a
development dependency.

Public packages must not require this private crate to build, package, publish,
or run. Product source must not call testkit helpers. The crate is excluded from
the crates.io publication order.

## Determinism Contract

Shared builders and fake adapters produce the same semantic object for the same
arguments. They do not read ambient environment, depend on wall-clock time, or
reuse mutable global state unless a test explicitly supplies that dependency.

Fixture lookup is rooted from a supplied manifest or workspace path. Missing,
malformed, or unknown evidence assets fail with the requested identity and
path; loaders do not substitute a nearby fixture.

## Assertion Contract

Assertions normalize only fields that the owning product contract declares
non-semantic. They must not delete meaningful order, identity, status,
diagnostics, or provenance merely to stabilize a test.

Corruption builders create a named, reviewable fault. Random damage without a
retained seed or expected failure class does not belong in a shared helper.

## Change Rules

- Add a shared helper only when at least two owned test surfaces need the same
  semantics.
- Keep package-specific setup in the owning package.
- Update all consumers when a canonical fixture changes.
- Do not use testkit defaults to conceal a newly required production field.
- Preserve the distinction between synthetic fixtures and release evidence.

## Verification

| Claim | Required evidence |
| --- | --- |
| private package boundary | workspace package-boundary and release-validation contracts |
| fixture loading | focused testkit unit tests and consuming fixture contracts |
| fake adapter behavior | testkit fake-adapter tests plus runtime adapter contracts |
| assertion semantics | consumers that exercise normalized manifests and traces |

Run:

```bash
cargo test --locked -p bijux-dag-testkit
```

A consuming package test is still required when a shared helper changes the
meaning of its setup or assertion.
