use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{parse_graph_strict, Graph, GraphError, SPEC_VERSION};

fn parse_graph(input: &str) -> Graph {
    parse_graph_strict(input).expect("parse graph")
}

#[test]
fn canonicalization_order_is_independent_from_node_and_edge_order() -> Result<(), GraphError> {
    let a = parse_graph(&format!(
        r#"{{
  "spec": "{}",
  "nodes": [
    {{"id":"b","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"b/out"}}],"params":{{"value":2}}}},
    {{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"a/out"}}],"params":{{"value":1}}}}
  ],
  "edges": [{{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"b","port":"in"}}}}]
}}"#,
        SPEC_VERSION
    ));
    let b = parse_graph(&format!(
        r#"{{
  "spec": "{}",
  "nodes": [
    {{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"a/out"}}],"params":{{"value":1}}}},
    {{"id":"b","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"b/out"}}],"params":{{"value":2}}}}
  ],
  "edges": [{{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"b","port":"in"}}}}]
}}"#,
        SPEC_VERSION
    ));

    assert_eq!(a.to_canonical_json()?, b.to_canonical_json()?);
    assert_eq!(a.graph_fingerprint()?, b.graph_fingerprint()?);
    Ok(())
}

#[test]
fn fingerprint_ignores_non_semantic_json_object_field_order() -> Result<(), GraphError> {
    let first = parse_graph(&format!(
        r#"{{
  "spec": "{}",
  "inputs": {{"alpha": 1, "beta": 2}},
  "nodes": [
    {{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"a/out"}}],"params":{{"x":1,"y":2}}}}
  ],
  "edges": []
}}"#,
        SPEC_VERSION
    ));
    let second = parse_graph(&format!(
        r#"{{
  "spec": "{}",
  "inputs": {{"beta": 2, "alpha": 1}},
  "nodes": [
    {{"id":"a","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"a/out"}}],"params":{{"y":2,"x":1}}}}
  ],
  "edges": []
}}"#,
        SPEC_VERSION
    ));

    assert_eq!(first.graph_fingerprint()?, second.graph_fingerprint()?);
    Ok(())
}
