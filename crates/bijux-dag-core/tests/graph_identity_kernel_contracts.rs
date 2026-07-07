use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{parse_graph_strict, Graph, SPEC_VERSION};

fn parse_graph(input: &str) -> Graph {
    parse_graph_strict(input).expect("parse graph")
}

fn parse_yaml_graph(input: &str) -> Graph {
    let value: serde_json::Value = serde_yaml::from_str(input).expect("parse yaml");
    parse_graph(&serde_json::to_string(&value).expect("yaml to json"))
}

fn strip_json_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    let bytes = input.as_bytes();
    let mut in_string = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            let mut backslashes = 0usize;
            let mut j = i;
            while j > 0 && bytes[j - 1] == b'\\' {
                backslashes += 1;
                j -= 1;
            }
            if backslashes % 2 == 0 {
                in_string = !in_string;
            }
            out.push('"');
            i += 1;
            continue;
        }
        if !in_string && b == b'/' && i + 1 < bytes.len() {
            let n = bytes[i + 1];
            if n == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if n == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

#[test]
fn identity_is_stable_across_whitespace_and_comment_only_changes() {
    let source_a = format!(
        r#"{{
  // graph comment
  "spec": "{}",
  "nodes": [
    {{"id":"n","kind":"const","outputs":[{{"name":"out","path":"n/out.txt"}}]}}
  ],
  "edges": []
}}"#,
        SPEC_VERSION
    );
    let source_b = format!(
        r#"{{"spec":"{}",/*block comment*/"nodes":[{{"id":"n","kind":"const","outputs":[{{"name":"out","path":"n/out.txt"}}]}}],"edges":[]}}"#,
        SPEC_VERSION
    );
    let g1 = parse_graph(&strip_json_comments(&source_a));
    let g2 = parse_graph(&strip_json_comments(&source_b));
    assert_eq!(g1.graph_id().unwrap(), g2.graph_id().unwrap());
}

#[test]
fn identity_is_stable_across_yaml_key_ordering_differences() {
    let yaml_a = format!(
        r#"
spec: {}
nodes:
  - id: n
    kind: const
    outputs:
      - name: out
        path: n/out.txt
edges: []
"#,
        SPEC_VERSION
    );
    let yaml_b = format!(
        r#"
nodes:
  - outputs:
      - path: n/out.txt
        name: out
    kind: const
    id: n
edges: []
spec: {}
"#,
        SPEC_VERSION
    );
    let g1 = parse_yaml_graph(&yaml_a);
    let g2 = parse_yaml_graph(&yaml_b);
    assert_eq!(g1.graph_id().unwrap(), g2.graph_id().unwrap());
}

#[test]
fn identity_normalizes_utf8_paths_and_unicode_node_names() {
    let nfc_name = "cafe\u{00E9}";
    let nfd_name = "cafe\u{0065}\u{0301}";
    let nfc_path = "data/r\u{00E9}sult.txt";
    let nfd_path = "data/r\u{0065}\u{0301}sult.txt";

    let g_nfc = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"{}","kind":"const","outputs":[{{"name":"out","path":"{}"}}]}}],"edges":[]}}"#,
        SPEC_VERSION, nfc_name, nfc_path
    ));
    let g_nfd = parse_graph(&format!(
        r#"{{"spec":"{}","nodes":[{{"id":"{}","kind":"const","outputs":[{{"name":"out","path":"{}"}}]}}],"edges":[]}}"#,
        SPEC_VERSION, nfd_name, nfd_path
    ));

    assert_eq!(g_nfc.graph_id().unwrap(), g_nfd.graph_id().unwrap());
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

fn make_random_graph(mut seed: u64, nodes: usize) -> Graph {
    let mut graph = parse_graph(&format!(r#"{{"spec":"{}","nodes":[],"edges":[]}}"#, SPEC_VERSION));

    for i in 0..nodes {
        let id = format!("n{:03}", i);
        graph.nodes.push(bijux_dag_core::Node {
            id,
            kind: bijux_dag_core::NodeKind::Const,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: Vec::new(),
            outputs: vec![bijux_dag_core::FileOutput::new("out".to_string(), format!("o/{i}.txt"))],
            params: bijux_dag_core::ParamValue::Literal(serde_json::json!({
                "value": (lcg(&mut seed) % 1000) as i64
            })),
            container: None,
            timeout_ms: None,
            resources: None,
            tags: Vec::new(),
            retry: bijux_dag_core::RetryPolicy::default(),
            cache: Default::default(),
            effects: Vec::new(),
            env_allowlist: Vec::new(),
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        });
    }

    for to in 1..nodes {
        for from in 0..to {
            if (lcg(&mut seed) % 4) == 0 {
                graph.edges.push(bijux_dag_core::Edge {
                    id: None,
                    kind: bijux_dag_core::EdgeKind::Data,
                    decision: None,
                    from: bijux_dag_core::PortRef {
                        node_id: format!("n{:03}", from),
                        port: "out".to_string(),
                    },
                    to: bijux_dag_core::PortRef {
                        node_id: format!("n{:03}", to),
                        port: format!("in{from}"),
                    },
                });
                graph.nodes[to].inputs.push(format!("in{from}"));
            }
        }
    }
    graph
}

#[test]
fn random_dag_identity_property_is_deterministic() {
    for seed in 1..60u64 {
        let g = make_random_graph(seed, 12);
        let a = g.graph_id().unwrap();
        let b = g.graph_id().unwrap();
        assert_eq!(a, b, "graph id must be deterministic for seed {seed}");
    }
}

#[test]
fn random_edge_permutation_property_keeps_identity() {
    for seed in 17..55u64 {
        let g = make_random_graph(seed, 10);
        let id = g.graph_id().unwrap();
        let mut permuted = g.clone();
        permuted.edges.reverse();
        permuted.nodes.reverse();
        assert_eq!(id, permuted.graph_id().unwrap(), "seed {seed}");
    }
}

#[test]
fn canonicalization_is_idempotent_property() {
    for seed in 101..151u64 {
        let g = make_random_graph(seed, 9);
        let once = g.canonicalize();
        let twice = once.canonicalize();
        assert_eq!(once.to_canonical_json().unwrap(), twice.to_canonical_json().unwrap());
        assert_eq!(once.graph_id().unwrap(), twice.graph_id().unwrap());
    }
}

#[test]
fn graph_schema_failure_corpus_rejects_invalid_inputs() {
    let fixtures = [
        include_str!("fixtures/schema_failures/unknown_top_level_key.json"),
        include_str!("fixtures/schema_failures/missing_nodes_array.json"),
    ];

    for payload in fixtures {
        assert!(parse_graph_strict(payload).is_err(), "schema failure fixture must be rejected");
    }
}
