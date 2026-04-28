use super::handle_plan_command;
use crate::commands::{Commands, DagCli, PlanCommands};
use crate::ExitCode;
use std::fs;
use std::path::PathBuf;

fn quiet_json_cli() -> DagCli {
    DagCli { json: true, quiet: true, command: Commands::Version }
}

fn write_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-routes","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");
    (dir, dag)
}

fn write_invalid_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-invalid.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"bad-plan","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/out"}],"params":{"value":"2"}}
          ],
          "edges":[]
        }"#,
    )
    .expect("write invalid graph");
    (dir, dag)
}

fn write_tagged_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-tagged.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-routes-tagged","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"tags":["critical"],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write tagged graph");
    (dir, dag)
}

#[test]
fn plan_explain_success_path_returns_success() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &PlanCommands::Explain { dag }).expect("plan explain");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_success_path_returns_success() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_plan_command(&cli, &PlanCommands::Diagnostics { dag }).expect("plan diagnostics");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_command_rejects_malformed_input_without_panic() {
    let cli = quiet_json_cli();
    let result = std::panic::catch_unwind(|| {
        handle_plan_command(
            &cli,
            &PlanCommands::Explain { dag: PathBuf::from("/no/such/graph.json") },
        )
    });
    assert!(result.is_ok(), "plan route should not panic on malformed input");
    assert!(result.expect("result").is_err());
}

#[test]
fn plan_diagnostics_rejects_malformed_input_without_panic() {
    let cli = quiet_json_cli();
    let result = std::panic::catch_unwind(|| {
        handle_plan_command(
            &cli,
            &PlanCommands::Diagnostics { dag: PathBuf::from("/no/such/graph.json") },
        )
    });
    assert!(result.is_ok(), "plan diagnostics should not panic on malformed input");
    assert!(result.expect("result").is_err());
}

#[test]
fn plan_concise_human_snapshot_is_stable() {
    let expected = "a: Execute via selected_by_default (selected, queue=default)\n\
b: Execute via selected_by_default (selected, queue=default)";
    let (_tmp, dag) = write_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let result = bijux_dag_runtime::build_planner_analysis(
        &graph,
        &bijux_dag_runtime::RuntimeConfig::default(),
        &bijux_dag_runtime::RuntimeConfig::default().selectors,
        &bijux_dag_runtime::PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("plan");
    let rendered = super::concise_plan_lines(&result).join("\n");
    assert_eq!(rendered, expected);
}

#[test]
fn plan_routes_support_replay_and_imported_bundle_shaped_graphs() {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"imported-replay","owners":[],"tags":["imported","replay"]},
          "nodes":[{"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"seed"}}],
          "edges":[]
        }"#,
    )
    .expect("write graph");
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &PlanCommands::Explain { dag }).expect("plan explain");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_error_flow_is_stable_for_invalid_graph() {
    let (_tmp, dag) = write_invalid_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &PlanCommands::Diagnostics { dag }).unwrap_err();
    assert_eq!(code, ExitCode::from(3));
}

#[test]
fn plan_explain_dump_flow_is_stable_for_valid_graph() {
    let (_tmp, dag) = write_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let result = bijux_dag_runtime::build_planner_analysis(
        &graph,
        &bijux_dag_runtime::RuntimeConfig::default(),
        &bijux_dag_runtime::RuntimeConfig::default().selectors,
        &bijux_dag_runtime::PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("plan");
    assert!(!result.plan.order.is_empty(), "plan ordering should not be empty");
    assert!(!result.plan_fingerprint.is_empty(), "planner analysis should expose a fingerprint");
}

#[test]
fn plan_diff_success_path_returns_success() {
    let (_base_dir, before) = write_graph_fixture();
    let (_tagged_dir, after) = write_tagged_graph_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_plan_command(&cli, &PlanCommands::Diff { before, after }).expect("plan diff");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_closure_returns_success_for_selected_leaf() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(
        &cli,
        &PlanCommands::Closure { dag, select: vec!["b".to_string()] },
    )
    .expect("plan closure");
    assert_eq!(code, ExitCode::SUCCESS);
}
