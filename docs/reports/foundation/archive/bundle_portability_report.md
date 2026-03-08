# Bundle Portability Report

Generated: 2026-03-08

## Portability surfaces

- export/import roundtrip and verify-only stability:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- line-ending and path portability:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
- origin-preserving import/export behavior:
  - `crates/bijux-dag-app/tests/run_dir_import_export_contract.rs`
  - `docs/reports/foundation/bundle_import_fidelity_explain.md`

## Benchmark and scorecard anchors

- `docs/reports/foundation/bundle_export_import_benchmarks.md`
- `docs/reports/foundation/bundle_import_export_verify_fsck_benchmarks.md`
- `docs/reports/foundation/portability_scorecard.md`
- `docs/reports/foundation/portability_determinism_scorecard.md`

## Current posture

- bundle portability behavior is covered by regression and smoke-style contract tests
- portability cost and reliability are visible through benchmark and scorecard reports
