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

use bijux_dag_runtime::{deterministic_schedule_order, ReadyNode};
use std::collections::BTreeMap;

#[test]
fn scheduler_determinism_is_stable_for_same_inputs() {
    let nodes = vec![
        ReadyNode { node_id: "a".to_string(), priority: 1, attempt: 1, ready_unix_ms: 1 },
        ReadyNode { node_id: "b".to_string(), priority: 1, attempt: 1, ready_unix_ms: 1 },
    ];
    let first = deterministic_schedule_order(nodes.clone(), &BTreeMap::new());
    let second = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(first, second);
}
