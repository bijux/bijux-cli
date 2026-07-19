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

use bijux_dag_runtime::{
    classify_failure, deterministic_schedule_order, replay_equivalent, ReadyNode,
    RuntimeFailureClass,
};
use std::collections::BTreeMap;

#[test]
fn engine_correctness_uses_deterministic_dispatch_contract() {
    let nodes = vec![
        ReadyNode { node_id: "load".to_string(), priority: 1, attempt: 1, ready_unix_ms: 2 },
        ReadyNode { node_id: "extract".to_string(), priority: 1, attempt: 1, ready_unix_ms: 2 },
    ];
    let order = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(order[0].node_id, "extract");
    assert_eq!(order[1].node_id, "load");
}

#[test]
fn failure_path_classification_is_explicit() {
    let class = classify_failure(false, false, true, false, false, false);
    assert_eq!(class, RuntimeFailureClass::DependencyFailure);
}

#[test]
fn deterministic_replay_requires_fingerprint_equivalence() {
    assert!(replay_equivalent("run-fp-1", "run-fp-1"));
    assert!(!replay_equivalent("run-fp-1", "run-fp-2"));
}
