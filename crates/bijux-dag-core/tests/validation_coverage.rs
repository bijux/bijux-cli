use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{
    parse_graph_strict, Edge, FileOutput, Graph, GraphMeta, Node, NodeKind, PortRef, Severity,
    SPEC_VERSION,
};

#[test]
fn validation_error_and_warning_coverage() {
    let expected_error_codes = [
        "E1001", "E1002", "E1003", "E1004", "E1007", "E1008", "E1009", "E1010", "E1011", "E1020",
        "E1021", "E1022", "E1023", "E1024", "E1025",
    ];

    for code in expected_error_codes {
        let graph = graph_for_code(code);
        let diags = graph.validate_with_warnings();
        assert!(diags.iter().any(|d| d.code == code), "expected diagnostic {code}");
    }

    let warning_graphs = ["W2001", "W2002"];
    for code in warning_graphs {
        let graph = graph_for_code(code);
        let diags = graph.validate_with_warnings();
        let mut found = false;
        for diag in diags {
            if diag.code == code && diag.severity == Severity::Warning {
                found = true;
                break;
            }
        }
        assert!(found, "expected warning {code}");
    }
}

#[test]
fn parse_strict_rejects_unknown_fields_and_mismatched_spec() {
    let mut graph = base_graph();
    graph.spec = "bijux-dag/v9.9".to_string();
    let bad = serde_json::to_string(&graph).unwrap();
    assert!(parse_graph_strict(&bad).is_err());

    let bad_text = r#"{"spec":"bijux-dag/v0.1","nodes":[],"edges":[],"nondeterminism_allowed":false,"bad":true}"#;
    assert!(serde_json::from_str::<Graph>(bad_text).is_err());

    graph.spec = SPEC_VERSION.to_string();
    let good = serde_json::to_string(&graph).unwrap();
    assert!(parse_graph_strict(&good).is_ok());
}

#[test]
fn graph_node_fingerprint_reordering_invariance() {
    let graph = base_graph();
    let baseline = graph.graph_fingerprint().unwrap();
    for seed in 0..32u64 {
        let mut g = base_graph();
        shuffle_nodes(&mut g.nodes, seed);
        let current = g.graph_fingerprint().unwrap();
        assert_eq!(baseline, current);
    }
}

#[test]
fn node_fingerprint_reorders_inputs_and_outputs() {
    let mut graph = base_graph();
    graph.nodes[0].inputs = vec!["b".to_string(), "a".to_string(), "c".to_string()];
    let base = graph.node_fingerprint(&graph.nodes[0]).unwrap();
    graph.nodes[0].inputs = vec!["c".to_string(), "a".to_string(), "b".to_string()];
    let reordered = graph.node_fingerprint(&graph.nodes[0]).unwrap();
    assert_eq!(base, reordered);
}

#[test]
fn canonical_json_stable_with_reordered_graph_elements() {
    let graph = base_graph();
    let canonical = graph.to_canonical_json().unwrap();
    for seed in 1..16u64 {
        let mut g = base_graph();
        shuffle_edges(&mut g.edges, seed);
        shuffle_nodes(&mut g.nodes, seed + 3);
        assert_eq!(canonical, g.to_canonical_json().unwrap());
    }
}

#[test]
fn topo_order_is_dependency_sensitive() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes: vec![
            build_node("a", vec![], "out"),
            build_node("b", vec!["in".to_string()], "out"),
            build_node("c", vec!["in".to_string()], "out"),
        ],
        edges: vec![
            Edge {
                from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
            },
            Edge {
                from: PortRef { node_id: "b".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "c".to_string(), port: "in".to_string() },
            },
        ],
    };
    let order = graph.topo_order().unwrap();
    assert_eq!(order, vec!["a", "b", "c"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
}

#[test]
fn property_acyclic_graph_detection() {
    for nodes in 1..10 {
        let graph = chain_graph(nodes);
        assert!(graph.topo_order().is_ok());
    }
}

#[test]
fn property_duplicate_id_detection() {
    for added in 2..8 {
        let mut graph = base_graph();
        for _ in 0..added {
            graph.nodes.push(build_node("dup", vec![], "out"));
        }
        let diags = graph.validate_with_warnings();
        assert!(diags.iter().any(|d| d.code == "E1001"));
    }
}

#[test]
fn property_edge_target_validation() {
    let mut graph = base_graph();
    graph.edges.push(Edge {
        from: PortRef { node_id: "source".to_string(), port: "out".to_string() },
        to: PortRef { node_id: "missing".to_string(), port: "in".to_string() },
    });
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1002"));
}

#[test]
fn property_param_reference_validation() {
    let mut graph = base_graph();
    graph.nodes[0].params = bijux_dag_core::ParamValue::Object(
        [(
            "missing".to_string(),
            bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: Some("missing_input".to_string()),
                node_output: None,
            }),
        )]
        .into_iter()
        .collect(),
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn env_and_effect_requirements_regression() {
    let mut graph = base_graph();
    graph.nodes[0].env_allowlist = vec!["HOME".to_string()];
    graph.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1010"));
}

fn base_graph() -> Graph {
    let mut nodes = vec![
        build_node("source", vec![], "out"),
        build_node("sink", vec!["in".to_string()], "out"),
    ];
    nodes[1].inputs = vec!["in".to_string()];
    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: Some(GraphMeta {
            name: "base".to_string(),
            description: None,
            owners: vec!["ops".to_string()],
            tags: vec!["ci".to_string()],
        }),
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes,
        edges: vec![Edge {
            from: PortRef { node_id: "source".to_string(), port: "out".to_string() },
            to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
        }],
    }
}

fn graph_for_code(code: &str) -> Graph {
    match code {
        "E1001" => {
            let mut g = base_graph();
            g.nodes.push(build_node("source", vec![], "out"));
            g
        }
        "E1002" => {
            let mut g = base_graph();
            g.edges.push(Edge {
                from: PortRef { node_id: "missing".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            });
            g
        }
        "E1003" => {
            let mut g = base_graph();
            g.nodes[1].inputs = vec!["in".to_string()];
            g.edges.push(Edge {
                from: PortRef { node_id: "source".to_string(), port: "missing".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            });
            g
        }
        "E1004" => {
            let g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: serde_json::Map::new(),
                nondeterminism_allowed: false,
                nodes: vec![
                    build_node("a", vec![], "out"),
                    build_node("b", vec!["in".to_string()], "out"),
                ],
                edges: vec![
                    Edge {
                        from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
                        to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
                    },
                    Edge {
                        from: PortRef { node_id: "b".to_string(), port: "out".to_string() },
                        to: PortRef { node_id: "a".to_string(), port: "in".to_string() },
                    },
                ],
            };
            g
        }
        "E1007" => {
            let mut g = base_graph();
            g.nodes.push(build_node("bad node", vec![], "out"));
            g
        }
        "E1008" => {
            let mut g = base_graph();
            g.nodes[0].outputs =
                vec![FileOutput { name: "same".to_string(), path: "out.txt".to_string() }];
            g.nodes[1].outputs =
                vec![FileOutput { name: "same".to_string(), path: "out.txt".to_string() }];
            g
        }
        "E1009" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].effects = vec![];
            g
        }
        "E1010" => {
            let mut g = base_graph();
            g.nodes[0].env_allowlist = vec!["HOME".to_string()];
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g
        }
        "E1011" => {
            let mut g = base_graph();
            g.nodes[0].retry.max_attempts = 1;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Clock];
            g
        }
        "E1020" => {
            let mut g = base_graph();
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "seed".to_string(),
                    bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                        graph_input: Some("missing_seed".to_string()),
                        node_output: None,
                    }),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1021" => {
            let mut g = base_graph();
            g.nodes.push(Node {
                id: "dep".to_string(),
                kind: NodeKind::Const,
                inputs: vec!["in".to_string()],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "dep/out.txt".to_string(),
                }],
                params: bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                    graph_input: None,
                    node_output: Some(bijux_dag_core::NodeOutputRef {
                        node_id: "ghost".to_string(),
                        path: "out".to_string(),
                    }),
                }),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
            });
            g
        }
        "E1022" => {
            let mut g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: serde_json::Map::new(),
                nondeterminism_allowed: false,
                nodes: vec![
                    build_node("source", vec!["in".to_string()], "out"),
                    build_node("sink", vec!["in".to_string()], "out"),
                ],
                edges: vec![Edge {
                    from: PortRef { node_id: "source".to_string(), port: "out".to_string() },
                    to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
                }],
            };
            g.nodes[0].params = bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: None,
                node_output: Some(bijux_dag_core::NodeOutputRef {
                    node_id: "sink".to_string(),
                    path: "out".to_string(),
                }),
            });
            g
        }
        "E1023" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Container;
            g
        }
        "E1024" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Container;
            g.nodes[0].container = Some(bijux_dag_core::ContainerSpec {
                image: "alpine".to_string(),
                argv: vec![],
                env_allowlist: vec![],
                workdir: None,
                engine: "bad-engine".to_string(),
            });
            g.nodes[1].effects = vec![bijux_dag_core::Effect::Filesystem];
            g
        }
        "E1025" => {
            let mut g = base_graph();
            g.nodes[0].outputs =
                vec![FileOutput { name: "out".to_string(), path: "../bad.out".to_string() }];
            g
        }
        "W2001" => {
            let mut g = base_graph();
            g.nodes.push(build_node("isolated", vec!["in".to_string()], "out"));
            g
        }
        "W2002" => {
            let g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: serde_json::Map::new(),
                nondeterminism_allowed: false,
                nodes: vec![
                    build_node("source", vec![], "out"),
                    build_node("orphan", vec!["in".to_string()], "out"),
                ],
                edges: vec![],
            };
            g
        }
        _ => base_graph(),
    }
}

fn build_node(id: &str, mut inputs: Vec<String>, name: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Const,
        inputs: std::mem::take(&mut inputs),
        outputs: vec![FileOutput { name: name.to_string(), path: format!("{id}/{name}.txt") }],
        params: bijux_dag_core::ParamValue::default(),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
    }
}

fn chain_graph(len: usize) -> Graph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for idx in 0..len {
        let id = format!("n{idx}");
        nodes.push(build_node(&id, vec![], "out"));
        if idx > 0 {
            edges.push(Edge {
                from: PortRef { node_id: format!("n{}", idx - 1), port: "out".to_string() },
                to: PortRef { node_id: id, port: "in".to_string() },
            });
        }
        if let Some(node) = nodes.get_mut(idx) {
            if idx > 0 {
                node.inputs = vec!["in".to_string()];
            }
        }
    }

    for e in &mut edges {
        e.to.port = "in".to_string();
    }

    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes,
        edges,
    }
}

fn shuffle_nodes<T>(items: &mut Vec<T>, seed: u64) {
    if items.len() < 2 {
        return;
    }
    let mut s = seed;
    for idx in (1..items.len()).rev() {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let j = (s % (idx as u64 + 1)) as usize;
        items.swap(idx, j);
    }
}

fn shuffle_edges(edges: &mut Vec<Edge>, seed: u64) {
    shuffle_nodes(edges, seed);
}
