# App Module Dependency Graph Report

```text
lib.rs -> routes/validate_routes.rs
lib.rs -> routes/plan_routes.rs
lib.rs -> routes/run_routes.rs
lib.rs -> routes/inspect_routes.rs
lib.rs -> routes/replay_routes.rs
lib.rs -> routes/diff_routes.rs
lib.rs -> routes/prove_verify_routes.rs
lib.rs -> routes/artifact_routes.rs
lib.rs -> routes/runs_routes.rs
lib.rs -> routes/surface_routes.rs
lib.rs -> routes/diagnostics_routes.rs
lib.rs -> routes/export_import_routes.rs
response.rs -> ExitCode
response.rs -> ExitCode
diff_routes.rs -> commands::DagCli
diff_routes.rs -> {emit_json, print_human_diff, replay_service, ExitCode}
export_import_routes.rs -> commands::DagCli
export_import_routes.rs -> {
export_import_routes.rs -> commands::{Commands, DagCli}
export_import_routes.rs -> ExitCode
inspect_routes.rs -> commands::DagCli
inspect_routes.rs -> routes::path_resolution::{manifest_path, node_outputs_index_path, node_trace_path}
inspect_routes.rs -> routes::run_lookup::read_manifest_json
inspect_routes.rs -> {emit_json, load_snapshot, read_file, read_node_traces, ExitCode}
inspect_routes.rs -> commands::{Commands, DagCli}
inspect_routes.rs -> ExitCode
run_routes.rs -> commands::{CacheModeArg, DagCli, MaterializeModeArg}
run_routes.rs -> routes::preconditions::require_file
run_routes.rs -> {emit_json, map_materialize_mode, parse_graph, parse_selectors, read_file, ExitCode}
validate_routes.rs -> commands::DagCli
validate_routes.rs -> {emit_json, parse_graph, read_file, ExitCode}
validate_routes.rs -> commands::DagCli
preconditions.rs -> ExitCode
replay_routes.rs -> commands::{CacheModeArg, DagCli, MaterializeModeArg}
replay_routes.rs -> graph_helpers::parse_selectors
replay_routes.rs -> replay_cmd::ReplayCommandResponse
replay_routes.rs -> {
artifact_routes.rs -> commands::DagCli
artifact_routes.rs -> {emit_json, inspect_artifact, ExitCode}
artifact_routes.rs -> commands::DagCli
prove_verify_routes.rs -> commands::DagCli
prove_verify_routes.rs -> routes::output_selection::{output_selection, OutputSelection}
prove_verify_routes.rs -> routes::replay_routes
prove_verify_routes.rs -> routes::response::simple_failure_payload
prove_verify_routes.rs -> {emit_json, verify_bundle_invariants, verify_run, ExitCode}
plan_routes_tests.rs -> commands::{Commands, DagCli, PlanCommands}
plan_routes_tests.rs -> ExitCode
surface_routes.rs -> capability_matrix::backend_capability_payload
surface_routes.rs -> commands::DagCli
surface_routes.rs -> replay_service
surface_routes.rs -> {emit_json, ExitCode}
surface_routes.rs -> commands::{Commands, DagCli}
surface_routes.rs -> ExitCode
diagnostics_routes.rs -> commands::DagCli
diagnostics_routes.rs -> emit_json
diagnostics_routes.rs -> replay_service
diagnostics_routes.rs -> routes::renderer::print_pretty_json
diagnostics_routes.rs -> commands::{Commands, DagCli}
diagnostics_routes.rs -> ExitCode
run_lookup.rs -> {read_file, read_run_id, ExitCode}
runs_routes.rs -> commands::{DagCli, RunsCommands}
runs_routes.rs -> inspect_service
runs_routes.rs -> {
runs_routes.rs -> commands::{Commands, DagCli, RunsCommands}
runs_routes.rs -> ExitCode
plan_routes.rs -> commands::{DagCli, PlanCommands}
plan_routes.rs -> {
output_selection.rs -> commands::DagCli
output_selection.rs -> commands::DagCli
```
