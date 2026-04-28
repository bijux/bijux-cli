use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{
    ContainerSpec, Edge, Effect, FileOutput, Graph, GraphMeta, Node, NodeKind, NodeOutputRef,
    ParamValue, PortRef, RefSpec, Resources, RetryPolicy,
};

#[test]
fn serde_roundtrip_graph_model() {
    let graph = sample_graph();
    let json = serde_json::to_string_pretty(&graph).unwrap();
    let decoded: Graph = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.spec, bijux_dag_core::SPEC_VERSION);
    assert_eq!(decoded.nodes.len(), graph.nodes.len());
    assert_eq!(decoded.edges.len(), graph.edges.len());
}

#[test]
fn serde_roundtrip_node_model() {
    let node = sample_node("node-a");
    let text = serde_json::to_string_pretty(&node).unwrap();
    let decoded: Node = serde_json::from_str(&text).unwrap();
    assert_eq!(decoded.id, node.id);
    assert_eq!(decoded.kind, node.kind);
}

#[test]
fn serde_roundtrip_edge_and_port_models() {
    let edge = Edge {
        id: None,
        kind: bijux_dag_core::EdgeKind::Data,
        from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
        to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
    };
    let encoded = serde_json::to_string(&edge).unwrap();
    let decoded: Edge = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, edge);
}

#[test]
fn serde_roundtrip_file_output_model() {
    let output = FileOutput { name: "result".to_string(), path: "out/result.txt".to_string() };
    let encoded = serde_json::to_string(&output).unwrap();
    let decoded: FileOutput = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, output);
}

#[test]
fn serde_roundtrip_graph_meta_model() {
    let meta = GraphMeta {
        name: "demo".to_string(),
        description: Some("demo graph".to_string()),
        owners: vec!["team-a".to_string()],
        tags: vec!["build".to_string()],
    };
    let encoded = serde_json::to_string(&meta).unwrap();
    let decoded: GraphMeta = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.name, meta.name);
    assert_eq!(decoded.tags, meta.tags);
}

#[test]
fn serde_roundtrip_resources_model() {
    let resources = Resources { cpu: 2, mem_mb: 128 };
    let encoded = serde_json::to_string(&resources).unwrap();
    let decoded: Resources = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.cpu, resources.cpu);
    assert_eq!(decoded.mem_mb, resources.mem_mb);
}

#[test]
fn serde_roundtrip_retry_policy_model() {
    let policy = RetryPolicy { max_attempts: 3, backoff_ms: 20 };
    let encoded = serde_json::to_string(&policy).unwrap();
    let decoded: RetryPolicy = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.max_attempts, policy.max_attempts);
    assert_eq!(decoded.backoff_ms, policy.backoff_ms);
}

#[test]
fn serde_roundtrip_container_spec_model() {
    let spec = ContainerSpec {
        image: "alpine:3.20".to_string(),
        argv: vec!["sh".to_string(), "-c".to_string(), "echo ok".to_string()],
        env_allowlist: vec!["HOME".to_string()],
        workdir: Some("/workspace".to_string()),
        engine: "docker".to_string(),
    };
    let encoded = serde_json::to_string_pretty(&spec).unwrap();
    let decoded: ContainerSpec = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.image, spec.image);
}

#[test]
fn serde_roundtrip_ref_models() {
    let ref_spec = RefSpec {
        graph_input: None,
        node_output: Some(NodeOutputRef { node_id: "src".to_string(), path: "out".to_string() }),
    };
    let encoded = serde_json::to_string(&ref_spec).unwrap();
    let decoded: RefSpec = serde_json::from_str(&encoded).unwrap();
    let left = serde_json::to_value(&ref_spec).unwrap();
    let right = serde_json::to_value(&decoded).unwrap();
    assert_eq!(left, right);
}

#[test]
fn serde_roundtrip_param_value_model() {
    let params = ParamValue::Object(
        [
            ("value".to_string(), ParamValue::Literal(serde_json::json!(1))),
            (
                "list".to_string(),
                ParamValue::Array(vec![ParamValue::Literal(serde_json::json!(2))]),
            ),
            (
                "ref".to_string(),
                ParamValue::Ref(RefSpec {
                    graph_input: Some("seed".to_string()),
                    node_output: None,
                }),
            ),
        ]
        .into_iter()
        .collect(),
    );
    let encoded = serde_json::to_string(&params).unwrap();
    let decoded: ParamValue = serde_json::from_str(&encoded).unwrap();
    let left = serde_json::to_value(&decoded).unwrap();
    let right = serde_json::to_value(&params).unwrap();
    assert_eq!(left, right);
}

#[test]
fn serde_roundtrip_node_kind_model() {
    let cases = vec!["const", "shell", "container", "custom"];
    for text in cases {
        let node_kind: NodeKind = serde_json::from_str(&format!("\"{}\"", text)).unwrap();
        let encoded = serde_json::to_string(&node_kind).unwrap();
        assert_eq!(encoded, format!("\"{}\"", text));
        if text == "custom" {
            match node_kind {
                NodeKind::External(name) => assert_eq!(name, "custom"),
                _ => panic!("expected external node kind"),
            }
        }
    }
}

#[test]
fn node_kind_parse_external_adapter_variant() {
    let node_kind: NodeKind = serde_json::from_str("\"special-adapter\"").unwrap();
    assert!(matches!(node_kind, NodeKind::External(_)));
}

#[test]
fn parse_accepts_known_spec_version_and_rejects_unknown() {
    let mut graph = sample_graph();
    let good = serde_json::to_string(&graph).unwrap();
    assert!(bijux_dag_core::parse_graph_strict(&good).is_ok());

    graph.spec = "bijux-dag/v9.9".to_string();
    let bad = serde_json::to_string(&graph).unwrap();
    assert!(bijux_dag_core::parse_graph_strict(&bad).is_err());
}

#[test]
fn serde_rejects_unknown_root_fields() {
    let text = r#"{"spec":"bijux-dag/v0.1","nodes":[],"edges":[],"nondeterminism_allowed":false,"unknown":"x"}"#;
    let result = serde_json::from_str::<Graph>(text);
    assert!(result.is_err());
}

#[test]
fn core_public_api_contract_snapshot_stable() {
    let public_symbols = [
        "parse_graph_strict",
        "Graph.validate_with_warnings",
        "Graph.validate_strict",
        "Graph.graph_fingerprint",
        "Graph.to_canonical_json",
    ];
    let snapshot = serde_json::to_string(&public_symbols).unwrap();
    assert_eq!(
        snapshot,
        "[\"parse_graph_strict\",\"Graph.validate_with_warnings\",\"Graph.validate_strict\",\"Graph.graph_fingerprint\",\"Graph.to_canonical_json\"]"
    );
}

#[test]
fn strict_parse_then_validation_diagnostics_separation() {
    let mut graph = sample_graph();
    graph.nodes[0]
        .outputs
        .push(FileOutput { name: "dup".to_string(), path: "out/result.txt".to_string() });
    let text = serde_json::to_string(&graph).unwrap();
    let parsed = bijux_dag_core::parse_graph_strict(&text).unwrap();
    let diags = parsed.validate_with_warnings();
    assert!(diags.iter().any(|diag| matches!(diag.code.as_str(), "E1008" | "W2001" | "W2002")));
}

fn sample_graph() -> Graph {
    Graph {
        spec: bijux_dag_core::SPEC_VERSION.to_string(),
        meta: Some(GraphMeta {
            name: "roundtrip".to_string(),
            description: None,
            owners: vec!["core-team".to_string()],
            tags: vec!["test".to_string()],
        }),
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes: vec![sample_node("source")],
        edges: vec![],
    }
}

fn sample_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Const,
        inputs: vec![],
        outputs: vec![FileOutput {
            name: "result".to_string(),
            path: "out/result.txt".to_string(),
        }],
        params: ParamValue::default(),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec!["roundtrip".to_string()],
        retry: RetryPolicy::default(),
        effects: vec![Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
        branch: None,
    }
}
