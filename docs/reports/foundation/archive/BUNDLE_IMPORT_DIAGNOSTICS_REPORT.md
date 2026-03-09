# Bundle Import Diagnostics Report

Generated: 2026-03-08

## Diagnostic surfaces

- truncated or malformed bundle rejection:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- missing required payload rejection and optional payload tolerance:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- corrupted payload rejection before acceptance:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- unsupported bundle-version rejection:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`

## Operational diagnostics

- fsck and verify-only integration:
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
  - `docs/reports/foundation/fsck_deep_verification_cost_report.md`
- completion contracts:
  - `crates/bijux-dev-dag/tests/bundle_portability_completion_contracts.rs`
  - `crates/bijux-dev-dag/tests/run_dir_import_export_hardening_contracts.rs`

## Current posture

- bundle-import failure classes are enforced through dedicated corruption and schema tests
- portability diagnostics remain visible via fsck/verify-only command surfaces
