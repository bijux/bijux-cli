use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{scheduler_contract_profile, Runtime, RuntimeConfig};
use bijux_dag_runtime::state_machine::{run_transition_allowed, RunLifecycleState};

#[test]
fn engine_flow_executes_minimal_graph_and_materializes_run_dir() {
    let graph_json = r#"
    {
      "spec":"v0.1",
      "nodes":[
        {
          "id":"seed",
          "kind":"const",
          "outputs":[{"name":"out","path":"out"}],
          "params":{"value":"ok"}
        }
      ],
      "edges":[]
    }"#;
    let graph = parse_graph_strict(graph_json).expect("parse graph");
    let out = tempfile::tempdir().expect("tmp");
    let runtime = Runtime::new();
    let run_dir = runtime
        .run(&graph, out.path(), RuntimeConfig::default())
        .expect("run succeeds");
    assert!(run_dir.join("manifest.json").exists());
    assert!(run_dir.join("graph.snapshot.json").exists());
}

#[test]
fn scheduler_profile_is_deterministic_and_lexicographic() {
    let profile = scheduler_contract_profile();
    let as_json = serde_json::to_value(profile).expect("serialize");
    assert_eq!(as_json["canonical_unit"], "node");
    assert_eq!(as_json["model"], "event_driven");
    assert_eq!(as_json["ready_tie_break"], "lexicographic_node_id");
}

#[test]
fn run_state_machine_guards_block_illegal_terminal_regressions() {
    assert!(run_transition_allowed(
        RunLifecycleState::Queued,
        RunLifecycleState::Ready
    ));
    assert!(!run_transition_allowed(
        RunLifecycleState::Succeeded,
        RunLifecycleState::Running
    ));
}
