use bijux_dag_core::{
    Edge, EdgeKind, FileOutput, Graph, Node, NodeKind, ParamValue, PortRef, RefSpec, RetryPolicy,
    SemanticNodeKind, TriggerRule, SPEC_VERSION,
};
use criterion as _;
use hex as _;
use serde as _;
use serde_json::Value;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

fn base_graph() -> Graph {
    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: std::collections::BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: std::collections::BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes: vec![
            Node {
                id: "a".to_string(),
                kind: NodeKind::Const,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
                trigger_rule: TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            },
            Node {
                id: "b".to_string(),
                kind: NodeKind::Const,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec!["in".to_string()],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
                trigger_rule: TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            },
        ],
        edges: vec![Edge {
            id: None,
            kind: EdgeKind::Data,
            decision: None,
            from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
            to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
        }],
    }
}

#[test]
fn fingerprint_stable_under_reorder() {
    let graph = base_graph();
    let mut reordered = base_graph();
    reordered.nodes.reverse();
    assert_eq!(graph.graph_fingerprint().unwrap(), reordered.graph_fingerprint().unwrap());
}

#[test]
fn canonicalization_stable_over_many_reorders() {
    let graph = base_graph();
    let canonical = graph.to_canonical_json().unwrap();
    let fingerprint = graph.graph_fingerprint().unwrap();
    for seed in 0..50u64 {
        let mut candidate = base_graph();
        shuffle_with_seed(&mut candidate.nodes[..], seed + 1);
        shuffle_with_seed(&mut candidate.edges[..], seed + 101);
        assert_eq!(candidate.to_canonical_json().unwrap(), canonical);
        assert_eq!(candidate.graph_fingerprint().unwrap(), fingerprint);
    }
}

#[test]
fn fingerprint_changes_on_param() {
    let mut graph = base_graph();
    graph.nodes[0].params = ParamValue::Object(
        [("x".to_string(), ParamValue::Literal(Value::from(1)))].into_iter().collect(),
    );
    let baseline = graph.graph_fingerprint().unwrap();
    graph.nodes[0].params = ParamValue::Object(
        [("x".to_string(), ParamValue::Literal(Value::from(2)))].into_iter().collect(),
    );
    assert_ne!(baseline, graph.graph_fingerprint().unwrap());
}

#[test]
fn canonicalize_stable_bytes() {
    let graph = base_graph();
    let mut last = graph.to_canonical_json().unwrap();
    for _ in 0..50 {
        let current = graph.to_canonical_json().unwrap();
        assert_eq!(last, current);
        last = current;
    }
}

#[test]
fn canonicalization_stable_under_random_ordering() {
    let graph = base_graph();
    let canonical = graph.to_canonical_json().unwrap();
    let fingerprint = graph.graph_fingerprint().unwrap();
    for seed in 1..25u64 {
        let mut candidate = base_graph();
        shuffle(&mut candidate.nodes, seed);
        shuffle(&mut candidate.edges, seed.wrapping_mul(7));
        assert_eq!(candidate.to_canonical_json().unwrap(), canonical);
        assert_eq!(candidate.graph_fingerprint().unwrap(), fingerprint);
    }
}

#[test]
fn resolver_determinism() {
    let mut graph = base_graph();
    graph.inputs.insert(
        "x".to_string(),
        bijux_dag_core::GraphInputSpec::from_default_value(serde_json::json!(1)).expect("spec"),
    );
    graph.nodes[0].params = ParamValue::Ref(RefSpec {
        graph_input: Some("x".to_string()),
        node_output: None,
        path_var: None,
    });
    let left = serde_json::to_string(&graph.resolve_graph().unwrap().resolved_params).unwrap();
    let right = serde_json::to_string(&graph.resolve_graph().unwrap().resolved_params).unwrap();
    assert_eq!(left, right);
}

fn shuffle_with_seed<T>(items: &mut [T], mut seed: u64) {
    if items.len() <= 1 {
        return;
    }
    for index in (1..items.len()).rev() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let swap_index = (seed % (index as u64 + 1)) as usize;
        items.swap(index, swap_index);
    }
}

fn shuffle<T>(items: &mut [T], mut seed: u64) {
    if items.len() <= 1 {
        return;
    }
    for index in (1..items.len()).rev() {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let swap_index = (seed as usize) % (index + 1);
        items.swap(index, swap_index);
    }
}
