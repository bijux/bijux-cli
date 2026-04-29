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

fn write_branch_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-branch.json");
    fs::write(&dag, bijux_dag_testkit::branch_semantics_graph_json()).expect("write branch graph");
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
fn plan_explain_payload_exposes_branch_contracts() {
    let (_tmp, dag) = write_branch_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read branch graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let result = bijux_dag_runtime::build_planner_analysis(
        &graph,
        &bijux_dag_runtime::RuntimeConfig::default(),
        &bijux_dag_runtime::RuntimeConfig::default().selectors,
        &bijux_dag_runtime::PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("plan");

    let payload = super::plan_explain_payload(&result);
    let planned_edges = payload["planned_edges"].as_array().expect("planned edges array");
    let branch_left = planned_edges
        .iter()
        .find(|edge| edge["id"].as_str() == Some("branch-left"))
        .expect("branch-left edge");
    let planned_nodes = payload["planned_nodes"].as_array().expect("planned nodes array");
    let branch_node = planned_nodes
        .iter()
        .find(|node| node["id"].as_str() == Some("decide"))
        .expect("branch planned node");

    assert_eq!(payload["planner_contract_version"], "bijux-dag-planner/v1");
    assert_eq!(payload["branch_paths"][0]["branch_node_id"], "decide");
    assert_eq!(payload["branch_paths"][0]["decision"], "left");
    assert_eq!(branch_left["kind"], "conditional");
    assert_eq!(branch_left["decision"], "left");
    assert_eq!(branch_node["semantic_kind"], "branch");
    assert_eq!(branch_node["trigger_rule"], "all_success");
    assert_eq!(branch_node["branch"]["decision_output"], "decision");
}

#[test]
fn plan_diff_success_path_returns_success() {
    let (_base_dir, before) = write_graph_fixture();
    let (_tagged_dir, after) = write_tagged_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &PlanCommands::Diff { before, after }).expect("plan diff");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_closure_returns_success_for_selected_leaf() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_plan_command(&cli, &PlanCommands::Closure { dag, select: vec!["b".to_string()] })
            .expect("plan closure");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_backfill_returns_success_for_partitioned_window() {
    let cli = quiet_json_cli();
    let code = handle_plan_command(
        &cli,
        &PlanCommands::Backfill {
            window_start_unix_ms: 100,
            window_end_unix_ms: 300,
            partition_key: vec!["sample-a".to_string(), "sample-b".to_string()],
        },
    )
    .expect("plan backfill");
    assert_eq!(code, ExitCode::SUCCESS);
}
