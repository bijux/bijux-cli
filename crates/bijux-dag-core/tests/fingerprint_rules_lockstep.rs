use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{parse_graph_strict, SPEC_VERSION};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn fingerprint_spec_lists_current_identity_rules() {
    let spec = fs::read_to_string(repo_root().join("docs/spec/FINGERPRINTS_v0.1.md"))
        .expect("read fingerprint spec");
    for token in [
        "Graph metadata is included",
        "`group` is excluded from node fingerprints",
        "Hash with SHA256",
        "canonical ordering",
    ] {
        assert!(
            spec.contains(token),
            "fingerprint spec missing required rule token: {token}"
        );
    }
}

#[test]
fn fingerprint_implementation_matches_documented_group_exclusion_and_meta_inclusion() {
    let graph_a = parse_graph_strict(&format!(
        r#"{{"spec":"{}","meta":{{"name":"dag-a"}},"nodes":[{{"id":"n","kind":"const","group":"x","outputs":[{{"name":"out","path":"p/out"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ))
    .expect("graph a");
    let graph_b = parse_graph_strict(&format!(
        r#"{{"spec":"{}","meta":{{"name":"dag-b"}},"nodes":[{{"id":"n","kind":"const","group":"y","outputs":[{{"name":"out","path":"p/out"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ))
    .expect("graph b");

    // Meta must affect graph identity.
    assert_ne!(graph_a.graph_id().unwrap(), graph_b.graph_id().unwrap());
    // Group must not affect node identity.
    assert_eq!(
        graph_a.node_fingerprint(&graph_a.nodes[0]).unwrap(),
        graph_b.node_fingerprint(&graph_b.nodes[0]).unwrap()
    );
}
