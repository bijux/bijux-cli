# App Route Responsibility Report

generated_from: `crates/bijux-dag-app/src/routes/*.rs`

## Command-family ownership

- run history, timeline, tree: `routes/runs_routes.rs`
- inspect and explain: `routes/inspect_routes.rs`
- planning: `routes/plan_routes.rs`
- diagnostics and trace helpers: `routes/diagnostics_routes.rs`
- capabilities, semantic portability, equivalence proof: `routes/surface_routes.rs`
- response envelope helpers: `routes/response.rs`
- rendering helpers: `routes/renderer.rs`
- path and preconditions support: `routes/path_resolution.rs`, `routes/preconditions.rs`, `routes/run_lookup.rs`

## Dispatch contract

- `crates/bijux-dag-app/src/lib.rs` remains command dispatch plus shared utilities.
