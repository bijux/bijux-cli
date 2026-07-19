use super::handle_plan_command;
use crate::commands::{AbsolutePathPolicyArg, Commands, DagCli, PlanCommands};
use crate::ExitCode;
use std::fs;
use std::path::PathBuf;

fn quiet_json_cli() -> DagCli {
    DagCli { json: true, quiet: true, command: Commands::Version }
}

fn explain_command(dag: PathBuf) -> PlanCommands {
    PlanCommands::Explain {
        dags: vec![dag],
        out: None,
        run_id: None,
        cache_dir: None,
        absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
        jobs: 1,
        cpu_budget: None,
        memory_budget_mb: None,
        gpu_device_budget: None,
        resource_capacity: Vec::new(),
        from_node: Vec::new(),
        to_node: Vec::new(),
        select: Vec::new(),
        exclude: Vec::new(),
        dependency_closure: false,
    }
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

fn write_selection_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-selection.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-selection","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}},
            {"id":"c","kind":"const","inputs":[],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":"3"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");
    (dir, dag)
}

fn write_execution_cost_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-cost.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-cost","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {
              "id":"b",
              "kind":"shell",
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","b"],"estimated_duration_ms":9000},
              "resources":{"cpu":4,"mem_mb":2048},
              "tags":["gpu:2"],
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":250},
              "cache":{"enabled":false,"reason":"network-bound"}
            },
            {
              "id":"c",
              "kind":"shell",
              "inputs":["left","right"],
              "outputs":[{"name":"out","path":"c/out"}],
              "params":{"argv":["echo","c"],"estimated_duration_ms":3000},
              "resources":{"cpu":2,"mem_mb":1024}
            }
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"left"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"right"}}
          ]
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

fn branch_semantics_graph_json() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"branch-contract","owners":[],"tags":[]},
      "nodes":[
        {"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"seed/out"}],"params":{"value":1}},
        {
          "id":"decide",
          "kind":"shell",
          "semantic_kind":"branch",
          "inputs":["in"],
          "outputs":[{"name":"decision","path":"decide/decision.txt"}],
          "effects":["filesystem"],
          "params":{"argv":["echo","left"]},
          "branch":{"decisions":["left","right"],"default_decision":"left","decision_output":"decision"}
        },
        {"id":"left","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":"left"},"trigger_rule":"any_success"},
        {"id":"right","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":"right"},"trigger_rule":"any_success"},
        {"id":"join","kind":"shell","inputs":["lhs"],"outputs":[{"name":"out","path":"join/out"}],"params":{"argv":["echo","join"]},"effects":["filesystem"]}
      ],
      "edges":[
        {"id":"seed-to-decide","from":{"node_id":"seed","port":"out"},"to":{"node_id":"decide","port":"in"}},
        {"id":"branch-left","kind":"conditional","decision":"left","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"left","port":"in"}},
        {"id":"branch-right","kind":"conditional","decision":"right","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"right","port":"in"}},
        {"id":"left-to-join","kind":"control","from":{"node_id":"left","port":"out"},"to":{"node_id":"join","port":"lhs"}}
      ]
    }"#
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

fn write_plan_diff_before_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-diff-before.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-diff-before","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","before"]},
              "resources":{"cpu":1,"mem_mb":64},
              "timeout_ms":1000,
              "retry":{"max_attempts":1,"backoff_ms":10}
            }
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write plan diff before graph");
    (dir, dag)
}

fn write_plan_diff_after_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-diff-after.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-diff-after","owners":[],"tags":[]},
          "nodes":[
            {"id":"c","kind":"const","outputs":[{"name":"out","path":"c/out"}],"params":{"value":2}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"result","path":"b/result.json"}],
              "params":{"argv":["echo","after"]},
              "resources":{"cpu":4,"mem_mb":256},
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":50}
            }
          ],
          "edges":[{"from":{"node_id":"c","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write plan diff after graph");
    (dir, dag)
}

fn write_plan_diff_metadata_only_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-diff-metadata-only.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-routes","description":"metadata-only change","owners":["ops"],"tags":["reviewed"]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write metadata-only diff graph");
    (dir, dag)
}

fn write_branch_graph_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-branch.json");
    fs::write(&dag, branch_semantics_graph_json()).expect("write branch graph");
    (dir, dag)
}

fn write_path_graph_fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-paths.json");
    let out = dir.path().join("runs");
    let cache_dir = dir.path().join("cache");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-routes-paths","owners":[],"tags":[]},
          "nodes":[
            {
              "id":"shell-copy",
              "kind":"shell",
              "inputs":[],
              "outputs":[{"name":"result","path":"result.txt"}],
              "params":{"argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"]}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write path graph");
    (dir, dag, out, cache_dir)
}

fn write_container_workdir_graph_fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph-container-workdir.json");
    let out = dir.path().join("runs");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-routes-container-workdir","owners":[],"tags":[]},
          "nodes":[
            {
              "id":"container-copy",
              "kind":"container",
              "outputs":[{"name":"result","path":"result.txt"}],
              "params":{},
              "container":{
                "image":"alpine:3.19",
                "argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"],
                "workdir":"/absolute/workdir",
                "engine":"docker"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write container workdir graph");
    (dir, dag, out)
}

#[test]
fn plan_explain_success_path_returns_success() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &explain_command(dag)).expect("plan explain");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_success_path_returns_success() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(&cli, &PlanCommands::Diagnostics { dags: vec![dag] })
        .expect("plan diagnostics");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_command_rejects_malformed_input_without_panic() {
    let cli = quiet_json_cli();
    let result = std::panic::catch_unwind(|| {
        handle_plan_command(&cli, &explain_command(PathBuf::from("/no/such/graph.json")))
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
            &PlanCommands::Diagnostics { dags: vec![PathBuf::from("/no/such/graph.json")] },
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
    let code = handle_plan_command(&cli, &explain_command(dag)).expect("plan explain");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_error_flow_is_stable_for_invalid_graph() {
    let (_tmp, dag) = write_invalid_graph_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_plan_command(&cli, &PlanCommands::Diagnostics { dags: vec![dag] }).unwrap_err();
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

    let payload = super::plan_explain_payload(
        &result,
        None,
        bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
    );
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
fn plan_explain_payload_reports_previewed_path_bindings() {
    let (_tmp, dag, out, cache_dir) = write_path_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let preview_layout =
        super::resolve_plan_preview_layout(Some(out.as_path()), Some("previewed")).expect("layout");
    let preview = super::PlanPreviewConfig {
        run_root: Some(out),
        run_id: preview_layout.as_ref().map(|layout| layout.run_id.clone()),
        cache_dir: Some(cache_dir),
        absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
        jobs: 1,
        cpu_budget: None,
        memory_budget_mb: None,
        gpu_device_budget: None,
        named_resource_capacities: std::collections::BTreeMap::new(),
        upstream_selection_targets: Vec::new(),
        downstream_selection_roots: Vec::new(),
        selectors: bijux_dag_runtime::SelectorSet::default(),
        dependency_closure: false,
    };
    let result = super::build_default_planner_analysis(&graph, &preview).expect("plan");
    let payload =
        super::plan_explain_payload(&result, preview_layout.as_ref(), preview.absolute_path_policy);

    assert_eq!(payload["run_layout"]["run_id"], "previewed");
    assert_eq!(payload["absolute_path_policy"], "allow_literal");
    let resolved_paths =
        payload["path_previews"][0]["resolved_paths"].as_array().expect("resolved paths");
    let resolved_argv =
        payload["path_previews"][0]["resolved_argv"].as_array().expect("resolved argv");
    assert_eq!(resolved_paths.len(), 2);
    assert_eq!(resolved_argv.len(), 3);
    assert_eq!(resolved_paths[0]["expression"], "{inputs_dir}/seed.txt");
    assert!(
        resolved_paths[0]["resolved_path"].as_str().is_some_and(
            |value| value.contains("/run.tmp-previewed/nodes/shell-copy/inputs/seed.txt")
        )
    );
    assert!(
        resolved_argv[1].as_str().is_some_and(
            |value| value.contains("/run.tmp-previewed/nodes/shell-copy/inputs/seed.txt")
        )
    );
    assert!(resolved_argv[2].as_str().is_some_and(
        |value| value.contains("/run.tmp-previewed/nodes/shell-copy/outputs/result.txt")
    ));
}

#[test]
fn plan_explain_rejects_literal_container_workdir_when_policy_denies_it() {
    let (_tmp, dag, out) = write_container_workdir_graph_fixture();
    let cli = quiet_json_cli();
    let error = handle_plan_command(
        &cli,
        &PlanCommands::Explain {
            dags: vec![dag],
            out: Some(out),
            run_id: Some("container-preview".to_string()),
            cache_dir: None,
            absolute_path_policy: AbsolutePathPolicyArg::DenyLiteral,
            jobs: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            resource_capacity: Vec::new(),
            from_node: Vec::new(),
            to_node: Vec::new(),
            select: Vec::new(),
            exclude: Vec::new(),
            dependency_closure: false,
        },
    )
    .expect_err("plan explain should fail");

    assert_eq!(error, ExitCode::from(3));
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
fn plan_equivalence_success_path_returns_success() {
    let (_base_dir, before) = write_graph_fixture();
    let (_tagged_dir, after) = write_tagged_graph_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_plan_command(&cli, &PlanCommands::Equivalence { before, after }).expect("equiv");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_diff_payload_reports_execution_affecting_categories() {
    let (_before_dir, before_path) = write_plan_diff_before_fixture();
    let (_after_dir, after_path) = write_plan_diff_after_fixture();
    let before_raw = fs::read_to_string(before_path).expect("read before graph");
    let after_raw = fs::read_to_string(after_path).expect("read after graph");
    let before_graph = crate::parse_graph(&before_raw).expect("before graph");
    let after_graph = crate::parse_graph(&after_raw).expect("after graph");
    let before_result =
        super::build_default_planner_analysis(&before_graph, &super::PlanPreviewConfig::default())
            .expect("before plan");
    let after_result =
        super::build_default_planner_analysis(&after_graph, &super::PlanPreviewConfig::default())
            .expect("after plan");
    let diff = bijux_dag_runtime::diff_plans(&before_result, &after_result);
    let payload = super::plan_diff_payload(&before_result, &after_result, &diff);

    assert_eq!(payload["changed"], true);
    assert_eq!(payload["diff"]["execution_affecting_changed"], true);
    assert_eq!(payload["diff"]["metadata_only_changed"], false);
    assert_eq!(payload["diff"]["added_nodes"], serde_json::json!(["c"]));
    assert_eq!(payload["diff"]["removed_nodes"], serde_json::json!(["a"]));
    assert_eq!(payload["diff"]["changed_params"], serde_json::json!(["b"]));
    assert_eq!(payload["diff"]["changed_outputs"], serde_json::json!(["b"]));
    assert_eq!(payload["diff"]["changed_resources"], serde_json::json!(["b"]));
    assert_eq!(payload["diff"]["changed_retry_timeout"], serde_json::json!(["b"]));
    assert_eq!(payload["diff"]["added_dependencies"], serde_json::json!(["data:-:-:c:out->b:in"]));
    assert_eq!(
        payload["diff"]["removed_dependencies"],
        serde_json::json!(["data:-:-:a:out->b:in"])
    );
}

#[test]
fn plan_diff_payload_classifies_metadata_only_change() {
    let (_before_dir, before_path) = write_graph_fixture();
    let (_after_dir, after_path) = write_plan_diff_metadata_only_fixture();
    let before_raw = fs::read_to_string(before_path).expect("read before graph");
    let after_raw = fs::read_to_string(after_path).expect("read after graph");
    let before_graph = crate::parse_graph(&before_raw).expect("before graph");
    let after_graph = crate::parse_graph(&after_raw).expect("after graph");
    let before_result =
        super::build_default_planner_analysis(&before_graph, &super::PlanPreviewConfig::default())
            .expect("before plan");
    let after_result =
        super::build_default_planner_analysis(&after_graph, &super::PlanPreviewConfig::default())
            .expect("after plan");
    let diff = bijux_dag_runtime::diff_plans(&before_result, &after_result);
    let payload = super::plan_diff_payload(&before_result, &after_result, &diff);

    assert_eq!(payload["changed"], true);
    assert_eq!(payload["diff"]["metadata_only_changed"], true);
    assert_eq!(payload["diff"]["execution_affecting_changed"], false);
    assert_eq!(payload["diff"]["changed_metadata"], serde_json::json!(["graph_meta"]));
    assert_eq!(payload["diff"]["added_nodes"], serde_json::json!([]));
    assert_eq!(payload["diff"]["changed_params"], serde_json::json!([]));
}

#[test]
fn plan_equivalence_payload_reports_metadata_drift_equivalence() {
    let (_before_dir, before_path) = write_graph_fixture();
    let (_after_dir, after_path) = write_tagged_graph_fixture();
    let before_raw = fs::read_to_string(before_path).expect("read before graph");
    let after_raw = fs::read_to_string(after_path).expect("read after graph");
    let before_graph = crate::parse_graph(&before_raw).expect("before graph");
    let after_graph = crate::parse_graph(&after_raw).expect("after graph");
    let before_result =
        super::build_default_planner_analysis(&before_graph, &super::PlanPreviewConfig::default())
            .expect("before plan");
    let after_result =
        super::build_default_planner_analysis(&after_graph, &super::PlanPreviewConfig::default())
            .expect("after plan");
    let report = bijux_dag_runtime::compare_plan_equivalence(&before_result, &after_result);
    let payload = super::plan_equivalence_payload(&before_result, &after_result, &report);

    assert_eq!(payload["equivalent"], true);
    assert_eq!(payload["report"]["equivalence_class"], "metadata_drift_equivalent");
    assert_eq!(payload["report"]["graph_identity_equal"], false);
    assert_eq!(payload["report"]["execution_fingerprint_equal"], true);
    assert_eq!(
        payload["report"]["ignored_non_execution_drift"],
        serde_json::json!(["graph_meta", "node_tags:a"])
    );
    assert_eq!(payload["report"]["non_equivalence_causes"], serde_json::json!([]));
}

#[test]
fn plan_equivalence_payload_reports_exact_non_equivalence_causes() {
    let (_before_dir, before_path) = write_plan_diff_before_fixture();
    let (_after_dir, after_path) = write_plan_diff_after_fixture();
    let before_raw = fs::read_to_string(before_path).expect("read before graph");
    let after_raw = fs::read_to_string(after_path).expect("read after graph");
    let before_graph = crate::parse_graph(&before_raw).expect("before graph");
    let after_graph = crate::parse_graph(&after_raw).expect("after graph");
    let before_result =
        super::build_default_planner_analysis(&before_graph, &super::PlanPreviewConfig::default())
            .expect("before plan");
    let after_result =
        super::build_default_planner_analysis(&after_graph, &super::PlanPreviewConfig::default())
            .expect("after plan");
    let report = bijux_dag_runtime::compare_plan_equivalence(&before_result, &after_result);
    let payload = super::plan_equivalence_payload(&before_result, &after_result, &report);

    assert_eq!(payload["equivalent"], false);
    assert_eq!(payload["report"]["equivalence_class"], "not_equivalent");
    assert_eq!(payload["report"]["execution_fingerprint_equal"], true);
    assert_eq!(
        payload["report"]["non_equivalence_causes"],
        serde_json::json!([
            "added_dependency:data:-:-:c:out->b:in",
            "added_node:c",
            "changed_outputs:b",
            "changed_params:b",
            "changed_resources:b",
            "changed_retry_timeout:b",
            "removed_dependency:data:-:-:a:out->b:in",
            "removed_node:a"
        ])
    );
}

#[test]
fn plan_closure_returns_success_for_selected_leaf() {
    let (_tmp, dag) = write_graph_fixture();
    let cli = quiet_json_cli();
    let code = handle_plan_command(
        &cli,
        &PlanCommands::Closure { dags: vec![dag], select: vec!["b".to_string()] },
    )
    .expect("plan closure");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn plan_explain_payload_surfaces_selector_and_omission_summary() {
    let (_tmp, dag) = write_selection_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let preview = super::PlanPreviewConfig {
        run_root: None,
        run_id: None,
        cache_dir: None,
        absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
        jobs: 1,
        cpu_budget: None,
        memory_budget_mb: None,
        gpu_device_budget: None,
        named_resource_capacities: std::collections::BTreeMap::new(),
        upstream_selection_targets: Vec::new(),
        downstream_selection_roots: Vec::new(),
        selectors: bijux_dag_runtime::SelectorSet {
            include: vec![bijux_dag_runtime::Selector::Id("b".to_string())],
            exclude: Vec::new(),
        },
        dependency_closure: true,
    };
    let result = super::build_default_planner_analysis(&graph, &preview).expect("plan");
    let payload = super::plan_explain_payload(&result, None, preview.absolute_path_policy);

    assert_eq!(payload["selection"]["requested_selectors"], serde_json::json!(["include:id:b"]));
    assert_eq!(payload["selection"]["dependency_closure_enabled"], true);
    assert_eq!(payload["selection"]["selected_nodes"], serde_json::json!(["a", "b"]));
    assert_eq!(payload["nodes"][0]["reason"], "selected_by_dependency_closure");
    assert_eq!(payload["nodes"][1]["reason"], "selected_by_include_selector");
    assert_eq!(
        payload["selection"]["omitted_nodes"],
        serde_json::json!([{ "node_id": "c", "reason": "not_selected_by_include_selector" }])
    );
}

#[test]
fn plan_explain_payload_surfaces_downstream_roots_and_closure_reasons() {
    let (_tmp, dag) = write_selection_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let preview = super::PlanPreviewConfig {
        run_root: None,
        run_id: None,
        cache_dir: None,
        absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
        jobs: 1,
        cpu_budget: None,
        memory_budget_mb: None,
        gpu_device_budget: None,
        named_resource_capacities: std::collections::BTreeMap::new(),
        upstream_selection_targets: Vec::new(),
        downstream_selection_roots: vec!["a".to_string()],
        selectors: bijux_dag_runtime::SelectorSet::default(),
        dependency_closure: false,
    };
    let result = super::build_default_planner_analysis(&graph, &preview).expect("plan");
    let payload = super::plan_explain_payload(&result, None, preview.absolute_path_policy);

    assert_eq!(payload["selection"]["downstream_roots"], serde_json::json!(["a"]));
    assert_eq!(payload["selection"]["requested_selectors"], serde_json::json!(["from-node:a"]));
    assert_eq!(payload["selection"]["selected_nodes"], serde_json::json!(["a", "b"]));
    assert_eq!(payload["nodes"][0]["reason"], "selected_by_from_node");
    assert_eq!(payload["nodes"][1]["reason"], "selected_by_downstream_closure");
    assert_eq!(
        payload["selection"]["omitted_nodes"],
        serde_json::json!([{ "node_id": "c", "reason": "not_selected_by_from_node" }])
    );
}

#[test]
fn plan_explain_payload_surfaces_upstream_targets_and_closure_reasons() {
    let (_tmp, dag) = write_selection_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let preview = super::PlanPreviewConfig {
        run_root: None,
        run_id: None,
        cache_dir: None,
        absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
        jobs: 1,
        cpu_budget: None,
        memory_budget_mb: None,
        gpu_device_budget: None,
        named_resource_capacities: std::collections::BTreeMap::new(),
        upstream_selection_targets: vec!["b".to_string()],
        downstream_selection_roots: Vec::new(),
        selectors: bijux_dag_runtime::SelectorSet::default(),
        dependency_closure: false,
    };
    let result = super::build_default_planner_analysis(&graph, &preview).expect("plan");
    let payload = super::plan_explain_payload(&result, None, preview.absolute_path_policy);

    assert_eq!(payload["selection"]["upstream_targets"], serde_json::json!(["b"]));
    assert_eq!(payload["selection"]["requested_selectors"], serde_json::json!(["to-node:b"]));
    assert_eq!(payload["selection"]["selected_nodes"], serde_json::json!(["a", "b"]));
    assert_eq!(payload["nodes"][0]["reason"], "selected_by_upstream_closure");
    assert_eq!(payload["nodes"][1]["reason"], "selected_by_to_node");
    assert_eq!(
        payload["selection"]["omitted_nodes"],
        serde_json::json!([{ "node_id": "c", "reason": "not_selected_by_to_node" }])
    );
}

#[test]
fn plan_explain_payload_surfaces_execution_cost_estimate() {
    let (_tmp, dag) = write_execution_cost_graph_fixture();
    let raw = fs::read_to_string(dag).expect("read graph");
    let graph = crate::parse_graph(&raw).expect("graph");
    let result =
        super::build_default_planner_analysis(&graph, &super::PlanPreviewConfig::default())
            .expect("plan");
    let payload = super::plan_explain_payload(
        &result,
        None,
        bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
    );

    assert_eq!(payload["execution_cost_estimate"]["node_count"], 3);
    assert_eq!(payload["execution_cost_estimate"]["root_nodes"], serde_json::json!(["a", "b"]));
    assert_eq!(payload["execution_cost_estimate"]["critical_path_length"], 2);
    assert_eq!(
        payload["execution_cost_estimate"]["critical_path"]["node_ids"],
        serde_json::json!(["b", "c"])
    );
    assert_eq!(payload["execution_cost_estimate"]["critical_path"]["total_duration_ms"], 12000);
    assert_eq!(payload["execution_cost_estimate"]["critical_path"]["estimated_duration_nodes"], 2);
    assert_eq!(
        payload["execution_cost_estimate"]["critical_path"]["unit_duration_fallback_nodes"],
        0
    );
    assert_eq!(payload["execution_cost_estimate"]["max_parallelism"], 2);
    assert_eq!(payload["execution_cost_estimate"]["demand"]["cpu_cores_total"], 7);
    assert_eq!(payload["execution_cost_estimate"]["demand"]["gpu_devices_total"], 2);
    assert_eq!(
        payload["execution_cost_estimate"]["cache_exposure"]["non_cacheable_node_ids"],
        serde_json::json!(["b"])
    );
    assert_eq!(payload["execution_cost_estimate"]["timeout_exposure"]["max_timeout_ms"], 5000);
    assert_eq!(payload["execution_cost_estimate"]["retry_exposure"]["max_attempts"], 3);
}

#[test]
fn plan_preview_config_preserves_runtime_budget_inputs() {
    let preview = super::PlanPreviewConfig {
        run_root: Some(PathBuf::from("/tmp/runs")),
        run_id: Some("planned-run".to_string()),
        cache_dir: Some(PathBuf::from("/tmp/cache")),
        absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
        jobs: 3,
        cpu_budget: Some(6),
        memory_budget_mb: Some(4096),
        gpu_device_budget: Some(2),
        named_resource_capacities: std::collections::BTreeMap::from([
            ("database_slot".to_string(), 2),
            ("license.render".to_string(), 1),
        ]),
        upstream_selection_targets: vec!["publish".to_string()],
        downstream_selection_roots: Vec::new(),
        selectors: bijux_dag_runtime::SelectorSet::default(),
        dependency_closure: true,
    };

    let config = super::default_analysis_runtime_config(&preview);

    assert_eq!(config.jobs, 3);
    assert_eq!(config.scheduler_policy.max_parallelism, 3);
    assert_eq!(config.cpu_budget, Some(6));
    assert_eq!(config.memory_budget_mb, Some(4096));
    assert_eq!(config.gpu_device_budget, Some(2));
    assert_eq!(config.named_resource_capacities.get("database_slot"), Some(&2));
    assert_eq!(config.named_resource_capacities.get("license.render"), Some(&1));
    assert_eq!(config.upstream_selection_targets, vec!["publish".to_string()]);
    assert!(config.partial_rerun_dependency_closure);
}

#[test]
fn plan_explain_accepts_composed_graph_fragments() {
    let dir = tempfile::tempdir().expect("tmp");
    let foundation = dir.path().join("foundation.json");
    let publication = dir.path().join("publication.json");
    fs::write(
        &foundation,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"report","path":"extract/report.json"}],"params":{"value":"seed"}}],
          "edges":[]
        }"#,
    )
    .expect("write foundation");
    fs::write(
        &publication,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"publish","kind":"const","inputs":["report"],"outputs":[{"name":"out","path":"publish/out.json"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}}],
          "edges":[{"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}]
        }"#,
    )
    .expect("write publication");

    let cli = quiet_json_cli();
    let code = handle_plan_command(
        &cli,
        &PlanCommands::Explain {
            dags: vec![foundation, publication],
            out: None,
            run_id: None,
            cache_dir: None,
            absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
            jobs: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            resource_capacity: Vec::new(),
            from_node: Vec::new(),
            to_node: Vec::new(),
            select: Vec::new(),
            exclude: Vec::new(),
            dependency_closure: false,
        },
    )
    .expect("plan explain");
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
