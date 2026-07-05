use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    build_backfill_plan, build_planner_analysis, build_replay_plan_annotations,
    compute_partial_run_closure, diff_plans, explain_plan, PlannerGuardrails, RuntimeConfig,
    Selector, SelectorSet,
};

fn sample_graph() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
        {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"tags":["noop"],"params":{"argv":["echo","b"]}},
        {"id":"c","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"argv":["echo","c"]}}
      ],
      "edges":[
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
      ]
    }"#
}

#[test]
fn planner_builds_phased_result_and_fingerprint() {
    let graph = parse_graph_strict(sample_graph()).expect("graph should parse");
    let options = RuntimeConfig::default();
    let result = build_planner_analysis(
        &graph,
        &options,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");
    assert!(!result.phases.is_empty());
    assert!(!result.plan_fingerprint.is_empty());
    assert!(!result.annotations.is_empty());
}

#[test]
fn planner_supports_closure_replay_backfill_diff_and_explain() {
    let graph = parse_graph_strict(sample_graph()).expect("graph should parse");
    let options = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("c".to_string())],
            exclude: vec![],
        },
        ..RuntimeConfig::default()
    };
    let before = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: false },
    )
    .expect("before planner should succeed");
    let after = build_planner_analysis(
        &graph,
        &options,
        &options.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after planner should succeed");
    let closure = compute_partial_run_closure(&after.plan, &["c".to_string()]);
    assert!(closure.contains("a"));
    let replay = build_replay_plan_annotations(&after.plan);
    assert!(!replay.is_empty());
    let backfill = build_backfill_plan(1, 10, vec!["p0".to_string(), "p1".to_string()]);
    assert_eq!(backfill.partition_keys.len(), 2);
    let diff = diff_plans(&before, &after);
    assert!(!diff.changed_annotations.is_empty() || !diff.changed_filter_reasons.is_empty());
    let explain = explain_plan(&after);
    assert!(!explain.phases.is_empty());
}

#[test]
fn planner_rejects_unsupported_runtime_capability_during_lowering() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let err = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect_err("planner must fail for unsupported capability requirements");
    assert!(err.contains("unsupported runtime capability"));
}

#[test]
fn planner_reports_path_previews_when_run_root_is_known() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"out","path":"out.txt"}],
              "params":{"argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"]},
              "effects":["filesystem"]
            },
            {
              "id":"container",
              "kind":"container",
              "outputs":[{"name":"out","path":"out.txt"}],
              "container":{
                "image":"alpine:3.20",
                "argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"],
                "workdir":"{work_dir}/scratch",
                "engine":"docker"
              },
              "effects":["filesystem"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let temp = tempfile::tempdir().expect("tmp");
    let result = build_planner_analysis(
        &graph,
        &RuntimeConfig {
            run_root: Some(temp.path().to_path_buf()),
            run_id: Some("preview".to_string()),
            ..RuntimeConfig::default()
        },
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let previews = result.path_previews.expect("path previews");
    let shell_preview =
        previews.iter().find(|preview| preview.node_id == "shell").expect("shell preview");
    assert_eq!(shell_preview.execution_surface, "host");
    assert_eq!(
        shell_preview.resolved_argv.as_ref().expect("shell argv"),
        &vec![
            "cp".to_string(),
            temp.path()
                .join("run.tmp-preview/nodes/shell/inputs/seed.txt")
                .display()
                .to_string(),
            temp.path()
                .join("run.tmp-preview/nodes/shell/outputs/result.txt")
                .display()
                .to_string(),
        ]
    );
    assert!(shell_preview
        .resolved_paths
        .iter()
        .any(|path| path.key_path == "$.argv[1]" && path.resolved_path.contains("/nodes/shell/inputs/seed.txt")));

    let container_preview = previews
        .iter()
        .find(|preview| preview.node_id == "container")
        .expect("container preview");
    assert_eq!(container_preview.execution_surface, "container");
    assert_eq!(
        container_preview.resolved_argv.as_ref().expect("container argv"),
        &vec![
            "cp".to_string(),
            "/bijux/node/inputs/seed.txt".to_string(),
            "/bijux/node/outputs/result.txt".to_string(),
        ]
    );
    assert!(container_preview
        .resolved_paths
        .iter()
        .any(|path| path.key_path == "container.workdir" && path.resolved_path == "/bijux/node/work/scratch"));
}

#[test]
fn planner_rejects_unresolved_container_command_templates() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"container",
              "kind":"container",
              "outputs":[{"name":"out","path":"out.txt"}],
              "params":{},
              "container":{
                "image":"alpine:3.20",
                "argv":["tool","{params.missing}"],
                "engine":"docker"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let err = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect_err("planner must reject unresolved container templates");
    assert!(err.contains("validation failed"));
}
