# CLI Stability and UX Status Report (341-360)

Generated: 2026-03-08

This report maps tasks 341-360 to CLI command-surface contracts, output stability checks,
error taxonomy coverage, smoke suites, and architectural guarantees.

## 341-353 CLI surface, compatibility, help/output, errors, ordering, latency, smoke, and no-panic coverage

- core CLI surface contracts:
  - `crates/bijux-dag-cli/tests/contract_surface.rs`
  - `crates/bijux-dag-cli/tests/taxonomy_and_policy_contracts.rs`
  - `crates/bijux-dag-cli/tests/cli_surface_completion_contracts.rs`
- smoke and route command coverage:
  - `crates/bijux-dag-cli/tests/smoke_pipeline.rs`
  - `crates/bijux-dag-app/tests/cli_validate_smoke_contract.rs`
- error/exit/output behavior:
  - `crates/bijux-dag-app/tests/error_exit_contract.rs`
  - `crates/bijux-dag-app/tests/error_output_contract.rs`
  - `crates/bijux-dag-app/tests/error_snapshot_contract.rs`

## 354-355 command inventory and usage heatmap reports

- `docs/reports/foundation/cli_command_inventory_report.md`
- `docs/reports/foundation/cli_command_usage_heatmap.md`

## 356 CLI compatibility verification suite

- `configs/suites/cli_stability_verification.json`

## 357 CLI stability dashboard

- `docs/reports/foundation/cli_stability_dashboard.md`

## 358 regression fixture pack

- `crates/bijux-dag-app/tests/snapshots/dag_command_tree.txt`
- `crates/bijux-dag-app/tests/snapshots/error_json_shape.json`

## 359 CLI error taxonomy report

- `docs/reports/foundation/cli_error_taxonomy_report.md`

## 360 ADR

- `docs/adr/20260308-cli-stability-guarantees.md`
