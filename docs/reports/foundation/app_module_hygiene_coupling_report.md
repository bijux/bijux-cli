# App Module Hygiene Coupling Report

generated_from: route delegation and boundary contract checks.

## Remaining direct route/business coupling to monitor

- `crates/bijux-dag-app/src/lib.rs` still contains shared utility logic and legacy helper surfaces.
- command-family behavior is delegated to route modules and should not regress into `lib.rs`.

## Coupling status

- route modules own command-family routing behavior.
- renderer and response helpers remain isolated under `routes/renderer.rs` and `routes/response.rs`.
- path and run lookup helpers remain isolated under `routes/path_resolution.rs` and `routes/run_lookup.rs`.

## Sources

- `crates/bijux-dev-dag/tests/app_router_responsibility_drift_contracts.rs`
- `docs/reports/foundation/app_route_coupling_report.md`
