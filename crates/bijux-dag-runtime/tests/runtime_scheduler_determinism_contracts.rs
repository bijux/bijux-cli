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
