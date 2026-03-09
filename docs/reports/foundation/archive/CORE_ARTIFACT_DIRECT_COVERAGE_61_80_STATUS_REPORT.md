# Core and Artifact Direct Coverage Status Report (61-80)

Generated: 2026-03-08

This status report maps tasks 61-80 to already-shipped direct tests, property tests,
fast suite coverage, and uncovered-path reporting.

## 61-71 direct module coverage

- Core graph/pipeline/planner direct coverage:
  - `crates/bijux-dag-core/tests/graph_pipeline_planner_expansion_contracts.rs`
  - `crates/bijux-dag-core/tests/direct_module_entrypoints_contracts.rs`
  - `crates/bijux-dag-core/tests/validation_entrypoints_contract.rs`
- Artifact IO/storage direct coverage:
  - `crates/bijux-dag-artifacts/tests/io_store_fs_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/storage_services_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`

## 72-73 graph identity/canonical property coverage

- `crates/bijux-dag-core/tests/graph_identity_property_contracts.rs`
- `crates/bijux-dag-core/tests/graph_pipeline_planner_expansion_contracts.rs`

## 74-78 artifact IO roundtrip/corruption coverage

- `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
- `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`

## 79 fast direct-coverage suite

- Suite: `configs/suites/core_artifact_direct_coverage_fast.json`
- Guard: `crates/bijux-dev-dag/tests/core_artifact_direct_coverage_fast_suite_contracts.rs`

## 80 still-weak product-path report

- `docs/reports/foundation/CORE_ARTIFACT_STILL_UNCOVERED_PRODUCT_PATHS_REPORT.md`
- Current status in scoped set: no remaining uncovered product paths.
