# Bundle Compatibility Dashboard

Generated: 2026-03-08

## Core compatibility signals

- export/import/fsck compatibility contracts:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
- completion and governance contracts:
  - `crates/bijux-dev-dag/tests/bundle_portability_completion_contracts.rs`
  - `crates/bijux-dev-dag/tests/run_dir_import_export_hardening_contracts.rs`

## Schema and drift signals

- schema governance anchors:
  - `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- bundle fixture inventory:
  - `docs/reports/foundation/BUNDLE_FIXTURE_INVENTORY_REPORT.md`

## Performance and portability signals

- `docs/reports/foundation/bundle_export_import_latency_report.md`
- `docs/reports/foundation/bundle_import_export_verify_fsck_benchmarks.md`
- `docs/reports/foundation/PORTABILITY_SCORECARD.md`

## Current status

- import compatibility and corruption rejection: covered
- replay compatibility through imported bundles: covered
- schema and portability observability: covered
