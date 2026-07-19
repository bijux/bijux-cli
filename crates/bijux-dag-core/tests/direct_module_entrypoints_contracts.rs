use bijux_dag_core::{
    canonical_json, canonicalize_graph, compile_graph, compile_graph_with_defaults,
    deterministic_topology_order, lower_graph_to_execution_plan, resolve_graph, validate_graph,
    validate_schema, validate_semantics, validate_topology, validation_rule_registry,
    EdgeDependencyKind, Graph, GraphDefaults, GraphError, PlanOptions, Severity, TypedEdge,
    ValidationDomain,
};
use criterion as _;
use hex as _;
use serde as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

fn parse_graph(input: &str) -> Graph {
    serde_json::from_str(input).expect("graph json")
}

#[test]
fn canonical_module_roundtrips_to_stable_json() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"a","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"value.txt"}]}],
          "edges":[]
        }"#,
    );

    let canonical = canonicalize_graph(&graph);
    let canonical_str = canonical_json(&canonical).expect("canonical json");
    assert!(canonical_str.contains("\"spec\": \"bijux-dag/v0.1\""));
    assert!(canonical_str.contains("\"id\": \"a\""));
}

#[test]
fn edge_module_converts_untyped_edges_to_data_dependencies() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"value.txt"}]},
            {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out.txt"]}}
          ],
          "edges":[{"from":{"node_id":"a","port":"value"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    );

    let typed = TypedEdge::from(graph.edges[0].clone());
    assert_eq!(typed.dependency, EdgeDependencyKind::Data);
    assert_eq!(typed.from.node_id, "a");
    assert_eq!(typed.to.node_id, "b");
}

#[test]
fn topology_module_returns_deterministic_order() -> Result<(), GraphError> {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"a.txt"}]},
            {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b.txt"}],"params":{"argv":["/bin/sh","-c","cat in > b.txt"]}}
          ],
          "edges":[{"from":{"node_id":"a","port":"value"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    );
    let order = deterministic_topology_order(&graph)?;
    assert_eq!(order, vec!["a".to_string(), "b".to_string()]);
    Ok(())
}

#[test]
fn validate_module_splits_domains_and_keeps_registry_stable() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"a","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"value.txt"}]}],
          "edges":[]
        }"#,
    );

    let registry = validation_rule_registry();
    assert!(!registry.is_empty(), "validation registry must be non-empty");
    assert!(registry.iter().any(|rule| matches!(rule.domain, ValidationDomain::Schema)));
    assert!(registry.iter().any(|rule| matches!(rule.domain, ValidationDomain::Semantic)));
    assert!(registry.iter().any(|rule| matches!(rule.domain, ValidationDomain::Topology)));

    let all = validate_graph(&graph);
    let errors: Vec<_> =
        all.iter().filter(|diag| matches!(diag.severity, Severity::Error)).collect();
    assert!(errors.is_empty(), "valid graph should not emit errors");
    assert!(validate_schema(&graph).is_empty());
    assert!(validate_semantics(&graph).is_empty());
    let topology = validate_topology(&graph);
    assert!(topology.iter().all(|diag| !matches!(diag.severity, Severity::Error)));
}

#[test]
fn validate_module_emits_severity_from_registered_rule_metadata() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"dup","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"a.txt"}]},
            {"id":"dup","kind":"const","params":{"value":"y"},"outputs":[{"name":"value","path":"b.txt"}]},
            {"id":"isolated","kind":"const","params":{"value":"z"},"outputs":[{"name":"value","path":"c.txt"}]}
          ],
          "edges":[]
        }"#,
    );

    let registry = validation_rule_registry();
    let diagnostics = validate_graph(&graph);

    for diagnostic in diagnostics {
        let rule = registry
            .iter()
            .find(|rule| rule.id == diagnostic.code)
            .expect("diagnostic must be backed by registered rule");
        assert_eq!(
            diagnostic.severity, rule.severity,
            "diagnostic severity drifted from registered rule {}",
            rule.id
        );
    }
}

#[test]
fn resolve_module_resolves_valid_graph() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"a","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"value.txt"}]}],
          "edges":[]
        }"#,
    );
    let resolved = resolve_graph(&graph).expect("resolve graph");
    assert_eq!(resolved.graph.nodes.len(), 1);
}

#[test]
fn planner_module_lowers_graph_to_execution_plan() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"source","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"source.txt"}]}
          ],
          "edges":[]
        }"#,
    );
    let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
    assert_eq!(plan.nodes.len(), 1);
    assert_eq!(plan.ordering, vec!["source".to_string()]);
    let source = plan.nodes.iter().find(|node| node.id == "source").expect("source node");
    assert!(source.deps.is_empty());
}

#[test]
fn compile_module_compiles_graph_without_contract_packaging() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"source","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"source.txt"}]}
          ],
          "edges":[]
        }"#,
    );
    let compiled = compile_graph(&graph).expect("compile graph");
    assert_eq!(compiled.normalized_graph.nodes.len(), 1);
    assert_eq!(compiled.plan_hints.deterministic_topology_order, vec!["source".to_string()]);
}

#[test]
fn compile_module_applies_defaults_without_contract_wrapper() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"source","kind":"const","params":{"value":"x"},"outputs":[{"name":"value","path":"source.txt"}]}
          ],
          "edges":[]
        }"#,
    );
    let defaults = GraphDefaults {
        retry: Some(bijux_dag_core::RetryPolicy { max_attempts: 3, backoff_ms: 10 }),
        resources: Some(bijux_dag_core::Resources {
            cpu: 1,
            mem_mb: 64,
            gpu_devices: 0,
            named_resources: std::collections::BTreeMap::new(),
        }),
    };
    let compiled = compile_graph_with_defaults(&graph, &defaults).expect("compile with defaults");
    let node = compiled
        .normalized_graph
        .nodes
        .iter()
        .find(|node| node.id == "source")
        .expect("source node");
    assert_eq!(node.retry.max_attempts, 3);
    assert_eq!(node.resources.as_ref().map(|resources| resources.cpu), Some(1));
}
