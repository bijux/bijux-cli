# Namespace Reservation Law

## Scope

Define immutable namespace reservation boundaries for runtime and plugin routing.

## Canonical sources

- `crates/bijux-cli/src/routing/contracts/product_mount.rs`
- `docs/constitution/official_product_namespace_registry.json`
- `docs/constitution/CLI_CONSTITUTION.md`

## Law

- Reserved runtime namespaces and known Bijux tool namespaces are immutable compatibility boundaries.
- Namespace normalization and case-folding must not permit takeover or shadowing.
- Reserved-path rejection must remain explicit and machine-readable.
- Plugin namespace validation and runtime route resolution must consume the same reserved namespace set.
