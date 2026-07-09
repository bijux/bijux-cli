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

use bijux_dag_core::{Graph, NodeKind};
use bijux_dag_runtime::{
    build_task_contract, validate_task_contracts, NodeProvenance, RuntimeConfig, TaskFailureReason,
    TaskIsolationMode, TaskResultEnvelope,
};
use serde_json::json;

fn fixture(path: &str) -> Graph {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../bijux-dag-core/tests/snapshots")
        .join(path);
    let text = std::fs::read_to_string(root).unwrap();
    bijux_dag_core::parse_graph_strict(&text).unwrap()
}

#[test]
fn node_execution_contract_supports_all_isolation_modes() {
    let mut graph = fixture("linear.dag.json");
    graph.nodes.push(bijux_dag_core::Node {
        id: "subprocess_mode".to_string(),
        kind: NodeKind::Shell,
        semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
        inputs: vec![],
        outputs: vec![bijux_dag_core::FileOutput::new(
            "out".to_string(),
            "subprocess_mode/out".to_string(),
        )],
        params: bijux_dag_core::ParamValue::Object(
            [(
                "argv".to_string(),
                bijux_dag_core::ParamValue::Array(vec![
                    bijux_dag_core::ParamValue::Literal(json!("/bin/sh")),
                    bijux_dag_core::ParamValue::Literal(json!("-c")),
                    bijux_dag_core::ParamValue::Literal(json!("echo ok")),
                ]),
            )]
            .into_iter()
            .collect(),
        ),
        container: None,
        timeout_ms: None,
        resources: Some(bijux_dag_core::Resources {
            cpu: 1,
            mem_mb: 128,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        }),
        tags: vec![],
        retry: bijux_dag_core::RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    });
    graph.nodes.push(bijux_dag_core::Node {
        id: "container_mode".to_string(),
        kind: NodeKind::Container,
        semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
        inputs: vec![],
        outputs: vec![bijux_dag_core::FileOutput::new(
            "out".to_string(),
            "container_mode/out".to_string(),
        )],
        params: bijux_dag_core::ParamValue::default(),
        container: Some(bijux_dag_core::ContainerSpec {
            image: "alpine:3.19".to_string(),
            argv: vec!["echo".to_string(), "ok".to_string()],
            env_allowlist: vec![],
            workdir: None,
            engine: "docker".to_string(),
        }),
        timeout_ms: None,
        resources: Some(bijux_dag_core::Resources {
            cpu: 1,
            mem_mb: 128,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        }),
        tags: vec![],
        retry: bijux_dag_core::RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    });
    graph.nodes.push(bijux_dag_core::Node {
        id: "external_mode".to_string(),
        kind: NodeKind::External("fake".to_string()),
        semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
        inputs: vec![],
        outputs: vec![bijux_dag_core::FileOutput::new(
            "out".to_string(),
            "external_mode/out".to_string(),
        )],
        params: bijux_dag_core::ParamValue::default(),
        container: None,
        timeout_ms: None,
        resources: Some(bijux_dag_core::Resources {
            cpu: 1,
            mem_mb: 128,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        }),
        tags: vec![],
        retry: bijux_dag_core::RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    });

    let options = RuntimeConfig::default();
    let contracts = validate_task_contracts(&graph, &options).unwrap();
    let kinds: Vec<_> = contracts.iter().map(|c| c.isolation_mode.clone()).collect();

    assert!(kinds.contains(&TaskIsolationMode::InProcess));
    assert!(kinds.contains(&TaskIsolationMode::Subprocess));
    assert!(kinds.contains(&TaskIsolationMode::Container));
    assert!(kinds.contains(&TaskIsolationMode::ExternalAdapter));
}

#[test]
fn python_nodes_use_subprocess_task_isolation() {
    let graph = Graph {
        spec: "bijux-dag/v0.1".to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![bijux_dag_core::Node {
            id: "python_mode".to_string(),
            kind: NodeKind::Python,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: vec![],
            outputs: vec![bijux_dag_core::FileOutput::new(
                "out".to_string(),
                "python_mode/out.json".to_string(),
            )],
            params: bijux_dag_core::ParamValue::Object(
                [
                    (
                        "module".to_string(),
                        bijux_dag_core::ParamValue::Literal(json!("demo_python_adapter")),
                    ),
                    ("function".to_string(), bijux_dag_core::ParamValue::Literal(json!("emit"))),
                ]
                .into_iter()
                .collect(),
            ),
            container: None,
            timeout_ms: None,
            resources: None,
            tags: vec![],
            retry: Default::default(),
            cache: Default::default(),
            effects: vec![bijux_dag_core::Effect::Filesystem],
            env_allowlist: vec![],
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        }],
        edges: vec![],
    };

    let contracts = validate_task_contracts(&graph, &RuntimeConfig::default()).unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].isolation_mode, TaskIsolationMode::Subprocess);
}

#[test]
fn http_nodes_use_in_process_task_isolation() {
    let graph = Graph {
        spec: "bijux-dag/v0.1".to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![bijux_dag_core::Node {
            id: "http_mode".to_string(),
            kind: NodeKind::Http,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: vec![],
            outputs: vec![bijux_dag_core::FileOutput::new(
                "response".to_string(),
                "http_mode/response.json".to_string(),
            )],
            params: bijux_dag_core::ParamValue::Object(
                [
                    ("method".to_string(), bijux_dag_core::ParamValue::Literal(json!("GET"))),
                    (
                        "url".to_string(),
                        bijux_dag_core::ParamValue::Literal(json!("https://example.test/health")),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            container: None,
            timeout_ms: None,
            resources: None,
            tags: vec![],
            retry: Default::default(),
            cache: Default::default(),
            effects: vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network],
            env_allowlist: vec![],
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        }],
        edges: vec![],
    };

    let contracts = validate_task_contracts(&graph, &RuntimeConfig::default()).unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].isolation_mode, TaskIsolationMode::InProcess);
}

#[test]
fn file_transform_nodes_use_in_process_task_isolation() {
    let graph = Graph {
        spec: "bijux-dag/v0.1".to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![bijux_dag_core::Node {
            id: "file_transform_mode".to_string(),
            kind: NodeKind::FileTransform,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: vec!["source".to_string()],
            outputs: vec![bijux_dag_core::FileOutput::new(
                "out".to_string(),
                "file_transform_mode/out.txt".to_string(),
            )],
            params: bijux_dag_core::ParamValue::Object(
                [
                    ("operation".to_string(), bijux_dag_core::ParamValue::Literal(json!("copy"))),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(json!("seed/source")),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            container: None,
            timeout_ms: None,
            resources: None,
            tags: vec![],
            retry: Default::default(),
            cache: Default::default(),
            effects: vec![bijux_dag_core::Effect::Filesystem],
            env_allowlist: vec![],
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        }],
        edges: vec![],
    };

    let contracts = validate_task_contracts(&graph, &RuntimeConfig::default()).unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].isolation_mode, TaskIsolationMode::InProcess);
}

#[test]
fn task_result_envelope_json_shape_is_stable() {
    let graph = fixture("linear.dag.json");
    let node = graph.nodes.first().unwrap().clone();
    let options = RuntimeConfig::default();
    let contract = build_task_contract(&node, &graph, &options);
    let envelope = TaskResultEnvelope {
        node_id: node.id.clone(),
        status: "success".to_string(),
        started_unix_ms: 1,
        finished_unix_ms: 2,
        attempts: 1,
        diagnostics: Vec::new(),
        effect_summary: vec!["filesystem".to_string()],
        outputs: vec!["a/out".to_string()],
        provenance: NodeProvenance {
            executable_identity: "runtime/local".to_string(),
            adapter_identity: "const@0.1".to_string(),
            resolved_task_contract: contract,
        },
        failure_reason: Some(TaskFailureReason::Execution),
    };
    let value = serde_json::to_value(&envelope).unwrap();
    assert_eq!(value["node_id"], json!(node.id));
    assert_eq!(value["status"], json!("success"));
    assert_eq!(value["attempts"], json!(1));
    assert_eq!(value["failure_reason"], json!("Execution"));
    assert_eq!(value["provenance"]["executable_identity"], json!("runtime/local"));
    assert_eq!(value["provenance"]["adapter_identity"], json!("const@0.1"));
    assert!(value["provenance"]["resolved_task_contract"].is_object());
}
