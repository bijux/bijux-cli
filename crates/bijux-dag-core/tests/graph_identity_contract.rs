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

fn parse_graph(payload: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(payload).expect("parse graph")
}

#[test]
fn graph_id_is_canonical_and_stable_under_non_semantic_reorder() {
    let a = parse_graph(&format!(
        r#"{{"spec":"{}","meta":{{"name":"x"}},"nodes":[{{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    let b = parse_graph(&format!(
        r#"{{"meta":{{"name":"x"}},"edges":[],"nodes":[{{"outputs":[{{"path":"a/out","name":"out"}}],"kind":"const","id":"a"}}],"spec":"{}"}}"#,
        SPEC_VERSION
    ));
    assert_eq!(a.graph_id().unwrap(), b.graph_id().unwrap());
}

#[test]
fn graph_id_changes_on_semantic_command_resource_and_env_changes() {
    let base = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"shell","params":{{"argv":["/bin/sh","-c","echo a"]}},"resources":{{"cpu":1,"mem_mb":64}},"env_allowlist":["A"],"outputs":[{{"name":"out","path":"n/out"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    let mut changed_cmd = base.clone();
    changed_cmd.nodes[0].params = bijux_dag_core::ParamValue::Literal(serde_json::json!({
        "argv": ["/bin/sh", "-c", "echo b"]
    }));
    let mut changed_res = base.clone();
    changed_res.nodes[0].resources = Some(bijux_dag_core::Resources {
        cpu: 2,
        mem_mb: 64,
        gpu_devices: 0,
        named_resources: std::collections::BTreeMap::new(),
    });
    let mut changed_env = base.clone();
    changed_env.nodes[0].env_allowlist.push("B".to_string());
    assert_ne!(base.graph_id().unwrap(), changed_cmd.graph_id().unwrap());
    assert_ne!(base.graph_id().unwrap(), changed_res.graph_id().unwrap());
    assert_ne!(base.graph_id().unwrap(), changed_env.graph_id().unwrap());
}

#[test]
fn graph_id_path_and_line_ending_normalization_contracts_hold() {
    let with_backslash = parse_graph(&format!(
        "{{\"spec\":\"{}\",\"nodes\":[{{\"id\":\"n\",\"kind\":\"const\",\"outputs\":[{{\"name\":\"out\",\"path\":\"dir\\\\out.txt\"}}]}}],\"edges\":[]}}",
        SPEC_VERSION
    ));
    let with_slash = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"const","outputs":[{{"name":"out","path":"dir/out.txt"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    assert_eq!(with_backslash.graph_id().unwrap(), with_slash.graph_id().unwrap());

    let lf = format!("{{\"spec\":\"{}\",\n\"nodes\":[],\n\"edges\":[]\n}}\n", SPEC_VERSION);
    let crlf = lf.replace('\n', "\r\n");
    let g_lf = parse_graph(&lf);
    let g_crlf = parse_graph(&crlf);
    assert_eq!(g_lf.graph_id().unwrap(), g_crlf.graph_id().unwrap());
}

#[test]
fn graph_id_unicode_paths_are_stable_when_canonicalized() {
    let g1 = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"const","outputs":[{{"name":"out","path":"ümlaut/数据/out.txt"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    let g2 = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"n","kind":"const","outputs":[{{"name":"out","path":"ümlaut/数据/out.txt"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    ));
    assert_eq!(g1.graph_id().unwrap(), g2.graph_id().unwrap());
}

#[test]
fn selected_subgraph_identity_is_deterministic() {
    let graph = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[
            {{"id":"a","kind":"const","outputs":[{{"name":"out","path":"a/out"}}]}},
            {{"id":"b","kind":"shell","inputs":["in"],"outputs":[{{"name":"out","path":"b/out"}}]}},
            {{"id":"c","kind":"shell","inputs":["in"],"outputs":[{{"name":"out","path":"c/out"}}]}}
        ],"edges":[
            {{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"b","port":"in"}}}},
            {{"from":{{"node_id":"a","port":"out"}},"to":{{"node_id":"c","port":"in"}}}}
        ]}}"#,
        SPEC_VERSION
    ));

    let mut sub_a = graph.clone();
    sub_a.nodes.retain(|n| n.id == "a" || n.id == "b");
    sub_a.edges.retain(|e| e.from.node_id != "a" || e.to.node_id != "c");

    let mut sub_b = sub_a.clone();
    sub_b.nodes.swap(0, 1);

    assert_eq!(sub_a.graph_id().unwrap(), sub_b.graph_id().unwrap());
}

#[test]
fn graph_fingerprint_explain_output_is_machine_readable() {
    let graph = parse_graph(&format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, SPEC_VERSION));
    let explain = graph.graph_fingerprint_explain().expect("explain");
    assert_eq!(explain.hash_algorithm, "sha256");
    assert!(!explain.graph_id.as_str().is_empty());
    assert!(explain.canonical_json_bytes_len > 0);
    let as_json = serde_json::to_value(&explain).expect("serialize explain");
    assert!(as_json["graph_id"].is_string());
}
