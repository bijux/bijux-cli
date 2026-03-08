# App Router Remaining Top-Level Responsibilities

generated_from: `crates/bijux-dag-app/src/lib.rs`

Top-level `lib.rs` responsibilities intentionally retained after route extraction:

1. Parse CLI arguments and dispatch command families.
2. Host shared utility helpers used across route and service modules.
3. Preserve stable command envelope behavior (`emit_json` and exit-code classification).
4. Keep compatibility-oriented helpers (`parse_graph`, snapshot/load helpers, shared readers).

Delegated command families:

- inspect: `routes/inspect_routes.rs`
- plan: `routes/plan_routes.rs`
- diagnostics: `routes/diagnostics_routes.rs`
- surface and capability: `routes/surface_routes.rs`
- artifact: `routes/artifact_routes.rs`
- replay/prove/verify/diff/export/import/run: dedicated route modules under `src/routes`

Policy and contract references:

- `configs/policy/app_routing_coverage_targets.json`
- `configs/policy/source_layout.json`
- `crates/bijux-dev-dag/tests/app_module_hygiene_contracts.rs`
- `crates/bijux-dev-dag/tests/app_routing_coverage_targets_contracts.rs`
