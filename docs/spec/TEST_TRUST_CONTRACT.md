# Test Trust Contract

## Scope

This contract defines the minimal repository surfaces that make runtime test
trust explicit instead of implicit.

## Required trust surfaces

- a human-readable testing philosophy document
- an architecture-facing audit page for the trust model
- a machine-readable runtime test trust catalog
- executable runtime and maintainer tests that keep the catalog current

## Catalog rules

`crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json` must group
runtime tests into non-empty trust classes, and every listed file must exist.

## Related tests

- `crates/bijux-dev/src/commands/ops.rs`
- `crates/bijux-dev/tests/test_trust_maintenance_contracts.rs`

## Versioning and change policy

Trust catalog structure and the requirement for explicit test classification are
stable contract surfaces. Any incompatible change requires updating this
document, the philosophy and audit pages, and the catalog in the same change.
