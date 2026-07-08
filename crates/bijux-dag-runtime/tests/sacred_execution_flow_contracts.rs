use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    run_transition_allowed, scheduler_contract_profile, ExecutionContext, NodeExecutionContext,
    NodeResult, RunLifecycleState, Runtime, RuntimeConfig,
};

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
    let run_dir = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run succeeds");
    assert!(run_dir.join("manifest.json").exists());
    assert!(run_dir.join("manifest.finalized.json").exists());
    assert!(run_dir.join(".run-complete.json").exists());
    assert!(run_dir.join("run.schema.json").exists());
    assert!(run_dir.join("graph.snapshot.json").exists());
}

#[test]
fn scheduler_profile_is_deterministic_and_lexicographic() {
    let profile = scheduler_contract_profile();
    let as_json = serde_json::to_value(profile).expect("serialize");
    assert_eq!(as_json["canonical_unit"], "node");
    assert_eq!(as_json["model"], "event_driven");
    assert_eq!(as_json["ready_tie_break"], "priority_cpu_memory_fit_then_node_id");
}

#[test]
fn run_state_machine_guards_block_illegal_terminal_regressions() {
    assert!(run_transition_allowed(RunLifecycleState::Queued, RunLifecycleState::Ready));
    assert!(!run_transition_allowed(RunLifecycleState::Succeeded, RunLifecycleState::Running));
}

#[test]
fn canonical_execution_context_and_result_surfaces_are_stable() {
    let _ = std::mem::size_of::<ExecutionContext>();
    let _ = std::mem::size_of::<NodeResult>();
    let _ = std::mem::size_of::<Option<NodeExecutionContext<'static>>>();
}

#[test]
fn engine_uses_centralized_sacred_hooks_without_direct_bypass_calls() {
    let source = std::fs::read_to_string("src/runtime_core/execution/engine.rs")
        .expect("engine source should exist");
    for required in [
        "sacred_execution::run_materialize_inputs",
        "sacred_execution::run_cache_lookup",
        "sacred_execution::run_retry_logic",
        "sacred_execution::run_write_trace",
        "sacred_execution::run_cache_write",
        "sacred_execution::resolve_dependencies",
    ] {
        assert!(source.contains(required), "engine missing sacred hook `{required}`");
    }
    for forbidden in [
        "crate::try_cache_read(",
        "crate::try_cache_write(",
        "crate::write_trace(",
        "crate::execute_with_retries(",
    ] {
        assert!(
            !source.contains(forbidden),
            "engine bypasses sacred hook with direct call `{forbidden}`"
        );
    }
}
