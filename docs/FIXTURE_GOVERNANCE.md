# Fixture governance

## Layout

- Compatibility fixtures are stored under `crates/bijux-dag-core/tests/compat/v0.1`.
- Runtime and CLI contract fixtures are embedded inside suite tests, not reused from root.

## Ownership rules

- Fixture churn requires fixture rationale in commit messages.
- Canonical files and fingerprints are treated as source-of-trust for compatibility assertions.
- Fixtures are not duplicated across top-level directories.

## Change policy

- Any fixture shape change must update associated tests and documentation contract in the same change.
