# App Route-Support Completion Report (421-440)

This report maps TODO 421-440 to direct route-support tests, route flow coverage, wording snapshots, and fast-lane support gates.

## 421-423 direct route-support tests

- `routes/output_selection.rs`: direct tests in module
- `routes/response.rs`: direct tests in module
- `routes/run_lookup.rs`: direct tests in module

## 424-430 inspect-related route flows

- listing/summary/timeline/tree/imported/corrupted route flows covered in:
  - `crates/bijux-dag-app/src/routes/runs_routes.rs` module tests

## 431-436 plan route flows

- plan explain/dump, diagnostics error, replay/import-shaped planning paths covered in:
  - `crates/bijux-dag-app/src/routes/plan_routes.rs` module tests

## 437-438 wording snapshots

- concise wording snapshot:
  - `crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs`
  - `crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt`
- detailed wording snapshot:
  - `crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs`
  - `crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt`

## 439 route-support coverage thresholds

- policy:
  - `configs/policy/app_routing_coverage_targets.json`
- contract:
  - `crates/bijux-dev-dag/tests/app_routing_coverage_targets_contracts.rs`

## 440 fast-lane route-support suite

- suite:
  - `configs/suites/app_route_support_fast.json`
- gate:
  - `crates/bijux-dev-dag/tests/app_route_support_fast_suite_contracts.rs`
