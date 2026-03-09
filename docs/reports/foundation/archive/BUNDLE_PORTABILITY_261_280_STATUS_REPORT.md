# Bundle Portability Hardening Status Report (261-280)

Generated: 2026-03-08

This report maps tasks 261-280 to bundle export/import/fsck tests, diagnostics,
compatibility surfaces, governance suites, and architectural guarantees.

## 261-270 export/import completeness, stability, idempotence, and corruption behavior

- export layout and payload shape coverage:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- reproducibility and stability across identical runs:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- import idempotence and verify-only behavior:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- corruption and version/schema rejection behavior:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
  - `docs/reports/foundation/BUNDLE_IMPORT_DIAGNOSTICS_REPORT.md`

## 271-274 fsck regression fixtures and replay compatibility

- bundle fixture and regression coverage:
  - `docs/reports/foundation/BUNDLE_FIXTURE_INVENTORY_REPORT.md`
- import/export replay compatibility coverage:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
  - `crates/bijux-dev-dag/tests/bundle_portability_completion_contracts.rs`

## 275-276 portability and import diagnostics reports

- `docs/reports/foundation/BUNDLE_PORTABILITY_REPORT.md`
- `docs/reports/foundation/BUNDLE_IMPORT_DIAGNOSTICS_REPORT.md`

## 277 bundle verification suite

- `configs/suites/bundle_portability_verification.json`

## 278 bundle schema drift detection

- `crates/bijux-dev-dag/tests/schema_governance_contracts.rs`
- `docs/reports/foundation/SCHEMA_CHANGELOG.md`

## 279 bundle compatibility dashboard

- `docs/reports/foundation/BUNDLE_COMPATIBILITY_DASHBOARD.md`

## 280 ADR

- `docs/adr/20260308-BUNDLE-PORTABILITY-GUARANTEES.md`
