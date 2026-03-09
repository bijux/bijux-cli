# App Route Weak Modules Completion Report

This report maps tasks 221-240 to direct route coverage and fast-lane governance artifacts.

## Direct route tests

- `crates/bijux-dag-app/src/routes/inspect_routes.rs`
- `crates/bijux-dag-app/src/routes/plan_routes.rs`
- `crates/bijux-dag-app/src/routes/diagnostics_routes.rs`
- `crates/bijux-dag-app/src/routes/output_selection.rs`
- `crates/bijux-dag-app/src/routes/surface_routes.rs`

## No-panic coverage

- inspect route entrypoints: explain, node, status
- plan route entrypoints: explain, diagnostics
- diagnostics route entrypoints: why-rerun, trace-artifact

## Concise human snapshot-style assertions

- inspect concise rendering: `concise_explain_human` in `inspect_routes.rs`
- plan concise rendering: `concise_plan_lines` in `plan_routes.rs`

## Fast suite for weakest route modules

- suite config: `configs/suites/app_weak_routes_fast.json`
- suite report: `docs/reports/foundation/app_weak_routes_fast_suite.md`
- contract gate: `crates/bijux-dev-dag/tests/app_weak_routes_fast_suite_contracts.rs`
