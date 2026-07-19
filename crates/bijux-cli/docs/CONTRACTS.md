# bijux-cli Contracts

Responsibility: Public command runtime, route normalization, plugin-aware execution, and structured output for the `bijux` binary.

## Scope

`bijux-cli` owns command parsing, route resolution, registry lookup, execution
flow, help rendering, interactive shell behavior, and machine-readable output
contracts for the public CLI runtime.

## Authority

This crate is authoritative for `bijux` command semantics, output envelopes,
plugin manifest consumption, and route normalization across binary and embedded
entrypoints.

## Invariants

- runtime behavior must stay deterministic across native and embedded launches
- help, registry lookup, and route normalization must agree on canonical paths
- plugin and product mounts must respect reserved namespace ownership
- DAG semantics and maintainer governance remain outside this crate boundary

## Related tests

- `crates/bijux-cli/tests/architecture.rs`
- `crates/bijux-cli/tests/integration.rs`
- `crates/bijux-cli/tests/routing.rs`

## Related schemas

- `contracts/official_product_namespace_registry.json`
- `contracts/product_mount_metadata_contract.json`
- `contracts/schemas/error-envelope-v1.schema.json`
- `contracts/schemas/output-envelope-v1.schema.json`
- `contracts/schemas/plugin-manifest-v2.schema.json`

## Versioning and change policy

Public command behavior, route normalization, output envelope shape, and plugin
compatibility are stable contract surfaces. Any incompatible change requires
updating this document, the linked contract inputs, and the related routing or
integration tests in the same change.
