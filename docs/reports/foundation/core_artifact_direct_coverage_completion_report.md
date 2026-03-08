# Core and Artifacts Direct Coverage Completion Report (281-300)

This report maps TODO 281-300 to implemented tests, suites, and generated evidence.

## 281-289 direct tests

- core graph and pipeline modules: direct contracts in `crates/bijux-dag-core/tests/`
- artifact io and storage modules: direct contracts in `crates/bijux-dag-artifacts/tests/`

## 290-292 property tests

- canonical-byte stability and graph identity properties:
  - `crates/bijux-dag-core/tests/graph_identity_property_contracts.rs`

## 293-298 artifact IO roundtrip and corruption tests

- roundtrip and corruption assertions:
  - `crates/bijux-dag-artifacts/tests/artifact_io_expansion_contracts.rs`
  - `crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs`

## 299 fast suite

- `configs/suites/core_artifact_direct_coverage_fast.json`
- guard: `crates/bijux-dev-dag/tests/core_artifact_direct_coverage_fast_suite_contracts.rs`

## 300 uncovered product-path report

- `docs/reports/foundation/core_artifact_still_uncovered_product_paths_report.md`
