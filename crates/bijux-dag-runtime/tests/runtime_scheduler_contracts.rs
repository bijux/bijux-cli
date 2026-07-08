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
fn scheduler_edge_case_empty_queue_is_stable() {
    let ordered = deterministic_schedule_order(Vec::new(), &BTreeMap::new());
    assert!(ordered.is_empty());
}

#[test]
fn scheduler_edge_case_equal_priority_uses_node_id_tie_break() {
    let nodes = vec![
        ReadyNode { node_id: "node-b".to_string(), priority: 1, attempt: 1, ready_unix_ms: 1 },
        ReadyNode { node_id: "node-a".to_string(), priority: 1, attempt: 1, ready_unix_ms: 1 },
    ];
    let ordered = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(ordered[0].node_id, "node-a");
}
