use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn planner_contract_covers_boundary_and_diagnostics_norms() {
    let root = repo_root();
    let planner_doc = fs::read_to_string(root.join("docs/spec/PLANNER_CONTRACT.md"))
        .expect("planner contract should exist");

    for required in [
        "parsed graph",
        "validated graph",
        "canonical graph",
        "execution plan",
        "P4000",
        "P4013",
        "P4016",
        "P4021",
        "dag plan-dump",
        "execution_plan.schema.json",
    ] {
        assert!(
            planner_doc.contains(required),
            "planner contract missing required token `{required}`"
        );
    }
}

#[test]
fn runtime_planner_bridge_uses_core_lowering_authority() {
    let root = repo_root();
    let runtime_planner = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/runtime_core/planning/planner.rs"),
    )
    .expect("runtime planner should exist");
    assert!(
        runtime_planner.contains("bijux_dag_core::lower_graph_to_execution_plan"),
        "runtime planner must delegate lowering to core planner boundary"
    );
}

#[test]
fn plan_truth_is_registered_as_battle_trust_property() {
    let root = repo_root();
    let policy_text = fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
        .expect("battle trust policy");
    let policy: serde_json::Value =
        serde_json::from_str(&policy_text).expect("battle trust policy parse");

    let trust_properties = policy
        .get("trust_properties")
        .and_then(serde_json::Value::as_array)
        .expect("trust properties array");
    assert!(
        trust_properties.iter().any(|entry| {
            entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == "tp_plan_truth")
        }),
        "battle trust policy must include tp_plan_truth"
    );
}
