# Execution Explainability Status Report (301-320)

Generated: 2026-03-08

This report maps tasks 301-320 to explain tests, schemas, snapshots, suites,
diagnostics outputs, and architectural guarantees.

## 301-314 explain determinism, corrupted/partial/imported/replay behavior, and reasoning coverage

- explain surface behavior and route wording contracts:
  - `crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs`
- advanced explainability regression coverage:
  - `crates/bijux-dev-dag/tests/advanced_explainability_completion_contracts.rs`
- app-level explain contracts:
  - `crates/bijux-dag-app/tests/diff_explain_contract.rs`
  - `crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs`
  - `crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs`

## 315-316 coverage and determinism reports

- `docs/reports/foundation/EXPLAIN_COVERAGE_REPORT.md`
- `docs/reports/foundation/EXPLAIN_DETERMINISM_REPORT.md`

## 317 explain verification suite

- `configs/suites/explain_verification.json`

## 318 explain diagnostics dashboard

- `docs/reports/foundation/EXPLAIN_DIAGNOSTICS_DASHBOARD.md`

## 319 explain operator smoke tests

- `configs/suites/explain_surface_stress.json`
- `configs/suites/advanced_explainability_regression.json`

## 320 ADR

- `docs/adr/20260308-EXPLAIN-SEMANTICS-GUARANTEES.md`
