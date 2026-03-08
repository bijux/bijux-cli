# Core and Artifacts Still-Uncovered Product Paths Report

Generated from direct test inventory in `crates/bijux-dag-core/tests` and `crates/bijux-dag-artifacts/tests`.

## Scope

- `crates/bijux-dag-core/src/graph/canonical.rs`
- `crates/bijux-dag-core/src/graph/edge.rs`
- `crates/bijux-dag-core/src/graph/topology.rs`
- `crates/bijux-dag-core/src/pipeline/validate.rs`
- `crates/bijux-dag-core/src/pipeline/resolve.rs`
- `crates/bijux-dag-artifacts/src/io/fs.rs`
- `crates/bijux-dag-artifacts/src/io/store.rs`
- `crates/bijux-dag-artifacts/src/storage/services.rs`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`

## Direct Coverage Anchors

- core graph canonical/edge/topology: `canonical_contract.rs`, `direct_module_entrypoints_contracts.rs`
- core pipeline validate/resolve: `validation_coverage.rs`, `validation_entrypoints_contract.rs`
- core identity properties: `graph_identity_property_contracts.rs`
- artifacts io fs/store: `io_store_fs_contracts.rs`, `artifact_io_expansion_contracts.rs`
- artifacts services/hardening: `storage_services_contracts.rs`, `artifact_storage_resilience_contracts.rs`

## Remaining Uncovered Product Paths

- none in this scoped set

## Fast Suite

- `configs/suites/core_artifact_direct_coverage_fast.json`
