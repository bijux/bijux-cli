# App Lib Direct Command Helpers

generated_from: `crates/bijux-dag-app/src/lib.rs`

Direct command/helper calls intentionally retained in top-level `lib.rs`:

- `show_effective_config`
- `show_effective_policy`
- `build_plan`
- `parse_graph`
- `read_file`
- `emit_json`

Most command-family behavior is routed through `src/routes/*`.
