use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{parse_graph_strict, SPEC_VERSION};

fn parse(text: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(text).expect("parse")
}

#[test]
fn node_group_does_not_affect_node_fingerprint() {
    let a = parse(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n1","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"x/out"}}],"group":"alpha"}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    let b = parse(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n1","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"x/out"}}],"group":"beta"}}],"edges":[]}}"#,
        SPEC_VERSION
    ));

    let fp_a = a.node_fingerprint(&a.nodes[0]).expect("node fp");
    let fp_b = b.node_fingerprint(&b.nodes[0]).expect("node fp");
    assert_eq!(fp_a, fp_b);
}

#[test]
fn graph_meta_fields_affect_graph_fingerprint() {
    let a = parse(&format!(
        r#"{{"spec":"{}","meta":{{"name":"dag_a"}},"nodes":[],"edges":[]}}"#,
        SPEC_VERSION
    ));
    let b = parse(&format!(
        r#"{{"spec":"{}","meta":{{"name":"dag_b"}},"nodes":[],"edges":[]}}"#,
        SPEC_VERSION
    ));

    assert_ne!(
        a.graph_fingerprint().expect("graph fp"),
        b.graph_fingerprint().expect("graph fp")
    );
}
