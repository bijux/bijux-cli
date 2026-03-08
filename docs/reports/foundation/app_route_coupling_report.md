# App Route Coupling Report

generated_from: static import and call graph inspection across route modules.

## Route coupling summary

- `routes/runs_routes.rs` couples to inspect service and response helpers.
- `routes/inspect_routes.rs` couples to inspect service and renderer helpers.
- `routes/plan_routes.rs` couples to planning helpers and renderer helpers.
- `routes/surface_routes.rs` couples to capability matrix and replay diff service only for equivalence proof assembly.

## Drift policy

- command-family business logic must stay in route/service modules.
- `lib.rs` must not directly host capability matrix decisions for command-family behavior.
