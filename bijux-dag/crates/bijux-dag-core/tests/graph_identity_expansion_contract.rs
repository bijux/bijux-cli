use bijux_dag_core::{
    canonical::canonical_json,
    edge::{EdgeDependencyKind, TypedEdge},
    parse_graph_strict,
    topology::deterministic_topology_order,
    validate::{validate_schema, validate_semantics, validate_topology, validation_rule_registry},
    Edge,
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

fn spec() -> &'static str {
    bijux_dag_core::SPEC_VERSION
}

#[test]
fn graph_identity_contract_covers_empty_disconnected_and_fan_shapes() {
    let empty = format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, spec());
    let disconnected = format!(
        r#"{{"spec":"{}","nodes":[{{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}},{{"id":"b","kind":"const","outputs":[{{"name":"out","path":"b/out"}}]}}],"edges":[]}}"#,
        spec()
    );
    let fan_out = format!(
        r#"{{"spec":"{}","nodes":[{{"id":"root","kind":"const","outputs":[{{"name":"out","path":"root/out"}}]}},{{"id":"left","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"left/out"}}]}},{{"id":"right","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"right/out"}}]}}],"edges":[{{"from":{{"node_id":"root","port":"out"}},"to":{{"node_id":"left","port":"in"}}}},{{"from":{{"node_id":"root","port":"out"}},"to":{{"node_id":"right","port":"in"}}}}]}}"#,
        spec()
    );
    let fan_in = format!(
        r#"{{"spec":"{}","nodes":[{{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}},{{"id":"b","kind":"const","outputs":[{{"name":"out","path":"b/out"}}]}},{{"id":"join","kind":"const","inputs":["a_in","b_in"],"outputs":[{{"name":"out","path":"join/out"}}]}}],"edges":[{{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"join","port":"a_in"}}}},{{"from":{{"node_id":"b","port":"out"}},"to":{{"node_id":"join","port":"b_in"}}}}]}}"#,
        spec()
    );

    for payload in [empty, disconnected, fan_out, fan_in] {
        let g = parse_graph_strict(&payload).expect("parse shape");
        let id_a = g.graph_id().expect("graph id");
        let id_b = g.graph_id().expect("graph id again");
        assert_eq!(id_a, id_b, "graph identity must be deterministic");
    }
}

#[test]
fn graph_id_changes_on_command_resource_env_and_output_path_mutations() {
    let base = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo hi"]}},"resources":{{"cpu":1,"mem_mb":64}},"env_allowlist":["A"],"outputs":[{{"name":"out","path":"n/out"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse base");
    let cmd = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo bye"]}},"resources":{{"cpu":1,"mem_mb":64}},"env_allowlist":["A"],"outputs":[{{"name":"out","path":"n/out"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse cmd");
    let res = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo hi"]}},"resources":{{"cpu":2,"mem_mb":64}},"env_allowlist":["A"],"outputs":[{{"name":"out","path":"n/out"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse res");
    let env = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo hi"]}},"resources":{{"cpu":1,"mem_mb":64}},"env_allowlist":["B"],"outputs":[{{"name":"out","path":"n/out"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse env");
    let out = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo hi"]}},"resources":{{"cpu":1,"mem_mb":64}},"env_allowlist":["A"],"outputs":[{{"name":"out","path":"n/other"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse output");

    assert_ne!(base.graph_id().unwrap(), cmd.graph_id().unwrap());
    assert_ne!(base.graph_id().unwrap(), res.graph_id().unwrap());
    assert_ne!(base.graph_id().unwrap(), env.graph_id().unwrap());
    assert_ne!(base.graph_id().unwrap(), out.graph_id().unwrap());
}

#[test]
fn canonical_and_topology_entrypoints_are_directly_covered() {
    let g = parse_graph_strict(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}},{{"id":"b","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"b/out"}}]}}],"edges":[{{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"b","port":"in"}}}}]}}"#,
        spec()
    ))
    .expect("parse");

    let canonical = canonical_json(&g).expect("canonical json");
    assert!(canonical.contains("\"nodes\""));
    let order = deterministic_topology_order(&g).expect("topology order");
    assert_eq!(order, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn edge_and_validate_entrypoints_are_directly_covered() {
    let edge = Edge {
        from: bijux_dag_core::PortRef {
            node_id: "a".to_string(),
            port: "out".to_string(),
        },
        to: bijux_dag_core::PortRef {
            node_id: "b".to_string(),
            port: "in".to_string(),
        },
    };
    let typed: TypedEdge = edge.into();
    assert_eq!(typed.dependency, EdgeDependencyKind::Data);

    let g = parse_graph_strict(&format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, spec()))
        .expect("parse");
    let rules = validation_rule_registry();
    assert!(!rules.is_empty());
    assert!(validate_schema(&g).is_empty());
    assert!(validate_semantics(&g).is_empty());
    assert!(validate_topology(&g).is_empty());
}

#[test]
fn graph_identity_is_stable_when_json_key_order_changes() {
    let a = parse_graph_strict(&format!(
        r#"{{"spec":"{}","meta":{{"name":"x"}},"nodes":[{{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}}],"edges":[]}}"#,
        spec()
    ))
    .expect("parse a");
    let b = parse_graph_strict(&format!(
        r#"{{"meta":{{"name":"x"}},"edges":[],"nodes":[{{"outputs":[{{"path":"a/out","name":"out"}}],"kind":"const","id":"a"}}],"spec":"{}"}}"#,
        spec()
    ))
    .expect("parse b");
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}
