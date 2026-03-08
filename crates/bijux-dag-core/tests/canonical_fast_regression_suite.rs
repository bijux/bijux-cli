use bijux_dag_core::parse_graph_strict;
use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

fn parse(input: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(input).expect("parse graph")
}

#[test]
fn canonical_fast_suite_keeps_identity_stable_for_non_semantic_reorder() {
    let a = parse(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"n/out.txt"}]}],"edges":[]}"#,
    );
    let b = parse(
        r#"{"nodes":[{"outputs":[{"path":"n/out.txt","name":"out"}],"kind":"const","id":"n"}],"edges":[],"spec":"bijux-dag/v0.1"}"#,
    );
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}

#[test]
fn canonical_fast_suite_detects_semantic_change() {
    let a = parse(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem"]}],"edges":[]}"#,
    );
    let b = parse(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","printf changed > out"]},"effects":["filesystem"]}],"edges":[]}"#,
    );
    assert_ne!(a.graph_id().unwrap(), b.graph_id().unwrap());
}

#[test]
fn canonical_fast_suite_normalizes_env_order() {
    let a = parse(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem","env"],"env_allowlist":["B","A"]}],"edges":[]}"#,
    );
    let b = parse(
        r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"n","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"n/out.txt"}],"params":{"argv":["/bin/sh","-c","cat in > out"]},"effects":["filesystem","env"],"env_allowlist":["A","B"]}],"edges":[]}"#,
    );
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}
