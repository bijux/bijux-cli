# CLI Error Taxonomy Report

Generated: 2026-03-08

## Error taxonomy classes

- validation input errors
- compatibility and unsupported-surface errors
- runtime execution and replay integrity errors
- schema and output-shape contract errors

## Contract anchors

- app/cli error and exit surfaces:
  - `crates/bijux-dag-app/tests/error_exit_contract.rs`
  - `crates/bijux-dag-app/tests/error_output_contract.rs`
  - `crates/bijux-dag-app/tests/error_snapshot_contract.rs`
- CLI compatibility and policy mapping:
  - `crates/bijux-dag-cli/tests/taxonomy_and_policy_contracts.rs`
  - `crates/bijux-dag-cli/tests/contract_surface.rs`

## Current posture

- CLI error classes are covered by explicit output and exit-code contracts
