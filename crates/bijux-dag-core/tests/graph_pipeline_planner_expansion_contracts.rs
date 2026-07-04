use bijux_dag_core::{
    canonical_json, deterministic_topology_order, lower_graph_to_execution_plan,
    parse_graph_strict, planner_identity_for_graph, resolve_graph, validate_graph,
    validate_topology, EdgeDependencyKind, Graph, GraphError, PlanOptions, Severity, TypedEdge,
};
use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

fn parse_graph(input: &str) -> Graph {
    parse_graph_strict(input).expect("graph json")
}

#[test]
fn canonical_minimal_fixture_is_stable() {
    let graph = parse_graph(r#"{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]}"#);
    let once = canonical_json(&graph).expect("canonical once");
    let twice = canonical_json(&parse_graph(&once)).expect("canonical twice");
    assert_eq!(once, twice);
}

#[test]
fn canonical_maximal_fixture_keeps_semantic_fields() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"maximal","owners":["ops"],"tags":["critical"]},
          "inputs":{"seed":123,"env":"prod"},
          "nondeterminism_allowed":false,
          "nodes":[
            {
              "id":"a",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"a/out"}],
              "params":{"argv":["/bin/sh","-c","cat in > out"]},
              "timeout_ms":1000,
              "resources":{"cpu":1,"mem_mb":128},
              "tags":["x"],
              "retry":{"max_attempts":2,"backoff_ms":10},
              "effects":["filesystem","env"],
              "env_allowlist":["A","B"],
              "group":"g"
            },
            {
              "id":"b",
              "kind":"const",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"value":1}
            }
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    );

    let canonical = canonical_json(&graph).expect("canonical");
    for needle in ["\"a\"", "\"b\"", "\"env_allowlist\"", "\"retry\""] {
        assert!(canonical.contains(needle), "missing {needle}");
    }
}

#[test]
fn legal_edge_construction_maps_to_data_dependency() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}]}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    );

    let typed = TypedEdge::from(graph.edges[0].clone());
    assert_eq!(typed.dependency, EdgeDependencyKind::Data);
}

#[test]
fn illegal_edge_construction_is_flagged_by_topology_validation() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}]}
          ],
          "edges":[{"from":{"node_id":"a","port":"missing"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    );

    let topology = validate_topology(&graph);
    assert!(
        topology.iter().any(|d| d.severity == Severity::Error),
        "expected topology validation errors for illegal edge port"
    );
}

#[test]
fn disconnected_graph_topology_is_deterministic() -> Result<(), GraphError> {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"z","kind":"const","outputs":[{"name":"out","path":"z/out"}]}
          ],
          "edges":[]
        }"#,
    );
    let order = deterministic_topology_order(&graph)?;
    assert_eq!(order, vec!["a".to_string(), "z".to_string()]);
    Ok(())
}

#[test]
fn fan_in_and_fan_out_topology_stays_stable() -> Result<(), GraphError> {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}]},
            {"id":"join","kind":"const","inputs":["a_in","b_in"],"outputs":[{"name":"out","path":"join/out"}]},
            {"id":"leaf1","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"leaf1/out"}]},
            {"id":"leaf2","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"leaf2/out"}]}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"join","port":"a_in"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"join","port":"b_in"}},
            {"from":{"node_id":"join","port":"out"},"to":{"node_id":"leaf1","port":"in"}},
            {"from":{"node_id":"join","port":"out"},"to":{"node_id":"leaf2","port":"in"}}
          ]
        }"#,
    );

    let order = deterministic_topology_order(&graph)?;
    assert_eq!(order, vec!["a", "b", "join", "leaf1", "leaf2"]);
    Ok(())
}

#[test]
fn validate_rejects_invalid_input_bindings() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out"}]},
            {"id":"sink","kind":"const","outputs":[{"name":"out","path":"sink/out"}],"params":{"value":{"graph_input":"missing"}}}
          ],
          "edges":[]
        }"#,
    );

    let diags = validate_graph(&graph);
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn validate_rejects_invalid_output_collisions() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"},{"name":"out","path":"a/out2"}]},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}]}
          ],
          "edges":[]
        }"#,
    );

    let diags = validate_graph(&graph);
    assert!(diags.iter().any(|d| d.code == "E1008"));
}

#[test]
fn resolve_is_deterministic_for_same_graph() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "inputs":{"seed":7},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":{"graph_input":"seed"}}}
          ],
          "edges":[]
        }"#,
    );

    let a = resolve_graph(&graph).expect("resolve a");
    let b = resolve_graph(&graph).expect("resolve b");
    assert_eq!(a.resolved_params, b.resolved_params);
}

#[test]
fn resolve_error_classification_surfaces_validation_failure() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":{"graph_input":"missing"}}}
          ],
          "edges":[]
        }"#,
    );

    let err = resolve_graph(&graph).expect_err("resolve must fail");
    assert!(matches!(err, GraphError::ValidationFailed));
}

#[test]
fn planner_imported_run_style_graph_is_plannable() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"hydrate_import","kind":"const","outputs":[{"name":"out","path":"hydrate_import/out"}],"params":{"value":"bundle-123"}},
            {"id":"replay_check","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"replay_check/out"}],"params":{"value":"verify"}}
          ],
          "edges":[{"from":{"node_id":"hydrate_import","port":"out"},"to":{"node_id":"replay_check","port":"in"}}]
        }"#,
    );

    let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
    assert_eq!(plan.ordering, vec!["hydrate_import", "replay_check"]);
}

#[test]
fn planner_selective_replay_keeps_dependency_closure() {
    let graph = parse_graph(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out"}]},
            {"id":"mid","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"mid/out"}]},
            {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}]}
          ],
          "edges":[
            {"from":{"node_id":"source","port":"out"},"to":{"node_id":"mid","port":"in"}},
            {"from":{"node_id":"mid","port":"out"},"to":{"node_id":"sink","port":"in"}}
          ]
        }"#,
    );

    let options = PlanOptions {
        selected_nodes: ["mid".to_string(), "sink".to_string()].into_iter().collect(),
        ..PlanOptions::default()
    };
    let plan = lower_graph_to_execution_plan(&graph, options).expect("plan");

    assert_eq!(plan.ordering, vec!["mid", "sink"]);
    assert_eq!(plan.edges.len(), 1);
    assert_eq!(plan.edges[0].from, "mid");
    assert_eq!(plan.edges[0].to, "sink");
}

#[test]
fn canonical_bytes_are_stable_under_ordering_variants() {
    let variants = [
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},{"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}]}],"edges":[]}"#,
        r#"{"nodes":[{"outputs":[{"path":"b/out","name":"out"}],"kind":"const","id":"b"},{"outputs":[{"path":"a/out","name":"out"}],"kind":"const","id":"a"}],"edges":[],"spec":"bijux-dag/v0.1"}"#,
    ];

    let ids: Vec<_> = variants
        .iter()
        .map(|payload| parse_graph(payload).graph_fingerprint().expect("fingerprint"))
        .collect();
    assert_eq!(ids[0], ids[1]);
}

#[test]
fn graph_identity_is_stable_across_default_normalization_variants() {
    let a = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out"}]}],"edges":[]}"#,
    );
    let b = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","inputs":[],"outputs":[{"name":"out","path":"n/out"}]}],"edges":[]}"#,
    );

    assert_eq!(a.graph_fingerprint().expect("a"), b.graph_fingerprint().expect("b"));
}

#[test]
fn graph_identity_is_stable_across_legacy_alias_normalization_paths() {
    let a = parse_graph(r#"{"spec":"0.1","nodes":[],"edges":[]}"#);
    let b = parse_graph(r#"{"spec":"v0.1","nodes":[],"edges":[]}"#);

    assert_eq!(a.graph_fingerprint().expect("a"), b.graph_fingerprint().expect("b"));
}

#[test]
fn graph_identity_changes_on_semantic_drift() {
    let a = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"]}],"edges":[]}"#,
    );
    let b = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out"}],"params":{"argv":["/bin/sh","-c","printf changed > out"]},"effects":["filesystem"]}],"edges":[]}"#,
    );

    assert_ne!(a.graph_fingerprint().expect("a"), b.graph_fingerprint().expect("b"));
}

#[test]
fn planner_identity_is_deterministic_for_same_input() {
    let graph = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out"}]}],"edges":[]}"#,
    );

    let first = planner_identity_for_graph(&graph).expect("planner identity a");
    let second = planner_identity_for_graph(&graph).expect("planner identity b");
    assert_eq!(first, second);
}

#[test]
fn planner_validation_errors_include_error_severity() {
    let graph = parse_graph(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":[]}],"edges":[]}"#,
    );

    let diags = validate_graph(&graph);
    assert!(diags.iter().any(|d| d.severity == Severity::Error));
}
