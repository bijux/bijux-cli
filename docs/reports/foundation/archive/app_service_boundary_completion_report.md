# App Service Boundary Completion Report

This report maps tasks 241-260 to implementation artifacts.

## Direct helper/module tests

- replay service and diff:
  - `crates/bijux-dag-app/src/replay/service.rs`
  - `crates/bijux-dag-app/src/replay/diff.rs`
- command helpers:
  - `crates/bijux-dag-app/src/commands/export_cmd.rs`
  - `crates/bijux-dag-app/src/commands/import_cmd.rs`
  - `crates/bijux-dag-app/src/commands/run_cmd.rs`
  - `crates/bijux-dag-app/src/commands/cli_model.rs`
  - `crates/bijux-dag-app/src/commands/config_resolution.rs`
  - `crates/bijux-dag-app/src/commands/config_surface.rs`
- cache helper:
  - `crates/bijux-dag-app/src/cache/cmd.rs`

## Service-boundary contracts

- `crates/bijux-dag-app/tests/service_boundary_contract.rs`

## Generated service-boundary reports

- `docs/reports/foundation/app_route_to_service_mapping.md`
- `docs/reports/foundation/app_lib_direct_command_helpers.md`
- `docs/reports/foundation/app_modules_zero_direct_tests_report.md`
- `docs/reports/foundation/app_modules_below_50_coverage_report.md`
- `docs/reports/foundation/app_hot_path_quality_dashboard.md`
