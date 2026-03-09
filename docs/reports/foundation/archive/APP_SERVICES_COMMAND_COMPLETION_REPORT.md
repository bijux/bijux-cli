# App Services and Command Tightening Completion Report (441-460)

This report maps TODO 441-460 to implemented tests, reports, and governance gates.

## 441-446 command module coverage

Command files requested in this range are not present as standalone modules in this crate:

- `commands/history_cmd.rs`
- `commands/inspect_cmd.rs`
- `commands/prove_verify_cmd.rs`
- `commands/artifact_cmd.rs`
- `commands/plan_cmd.rs`
- `commands/diagnostics_cmd.rs`

Equivalent command-family routing is covered through route modules and service contracts.

## 447-450 service boundaries

- inspect command -> inspect service boundary checks:
  - `crates/bijux-dag-app/tests/service_boundary_contract.rs`
- replay command -> replay service boundary checks:
  - `crates/bijux-dag-app/tests/service_boundary_contract.rs`
- diff command -> replay/diff service checks:
  - `crates/bijux-dag-app/tests/service_boundary_contract.rs`
- export/import command -> export/import helper checks:
  - `crates/bijux-dag-app/tests/service_boundary_contract.rs`

## 451-457 app service and command support checks

- prove/verify summary and portability/capability pathways covered by route and service tests already present.
- config-to-command effective-policy shaping covered by config command contract tests.
- app graph input precedence and workspace-root helper usage remain covered by graph input loading and run history contracts.

## 458-459 generated reports

- `docs/reports/foundation/APP_SERVICES_BOUNDARY_BYPASS_REPORT.md`
- `docs/reports/foundation/APP_MODULE_HYGIENE_COUPLING_REPORT.md`

## 460 ownership-class release gate

- policy:
  - `configs/policy/app_module_ownership_classes.json`
- contract:
  - `crates/bijux-dev-dag/tests/app_module_ownership_class_contracts.rs`
