use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    compatibility_matrix_report, compute_task_contract_fingerprint, default_task_type_registry,
    generate_task_contract_markdown, validate_cross_node_compatibility,
    validate_parameter_defaults, AdapterCapabilityDeclaration, TaskContract,
};
use std::collections::BTreeMap;

fn read_contract_fixture(name: &str) -> TaskContract {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/task_contract_conformance")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse fixture {}: {err}", path.display()))
}

#[test]
fn type_registry_and_default_validation_are_stable() {
    let registry = default_task_type_registry();
    assert!(!registry.scalar_types.is_empty());
    let contract = read_contract_fixture("shell.json");
    let diagnostics = validate_parameter_defaults(&contract, &BTreeMap::new());
    assert!(!diagnostics.is_empty());
}

#[test]
fn cross_node_compatibility_and_fingerprint_are_stable() {
    let producer = read_contract_fixture("const.json");
    let consumer = read_contract_fixture("shell.json");
    let diagnostics = validate_cross_node_compatibility(&producer, &consumer);
    assert!(diagnostics.is_empty());
    let fingerprint =
        compute_task_contract_fingerprint(&producer).expect("fingerprint should compute");
    assert_eq!(fingerprint.node_id, producer.node_id);
    let markdown = generate_task_contract_markdown(&consumer);
    assert!(markdown.contains("Task contract"));
}

#[test]
fn compatibility_matrix_report_is_generated_for_graph_snapshot() {
    let graph_text = r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {"id":"const-source","kind":"const","inputs":[],"outputs":[{"name":"out","path":"const/out.json"}],"params":{"value":1}},
        {"id":"shell-transform","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"shell/out.json"}],"params":{"argv":["echo","ok"]}}
      ],
      "edges":[
        {"from":{"node_id":"const-source","port":"out"},"to":{"node_id":"shell-transform","port":"in"}}
      ]
    }"#;
    let graph = parse_graph_strict(graph_text).expect("graph parse should pass");
    let mut contracts = BTreeMap::new();
    contracts.insert(
        "const-source".to_string(),
        read_contract_fixture("const.json"),
    );
    contracts.insert(
        "shell-transform".to_string(),
        read_contract_fixture("shell.json"),
    );
    let report = compatibility_matrix_report(&graph, &contracts);
    assert_eq!(report.relationships.len(), 1);
    assert!(report.relationships[0].compatible);
}

#[test]
fn adapter_capability_declaration_supports_replay_checks() {
    let declaration = AdapterCapabilityDeclaration {
        adapter_id: "shell".to_string(),
        adapter_version: "0.1".to_string(),
        supports_types: vec!["artifact_ref".to_string()],
        supports_replay_compatibility_check: true,
    };
    assert!(declaration.supports_replay_compatibility_check);
}
