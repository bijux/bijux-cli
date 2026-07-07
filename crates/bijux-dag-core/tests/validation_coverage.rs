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
    parse_graph_strict, BranchSpec, CacheBehavior, Edge, EdgeKind, FileOutput, Graph, GraphMeta,
    Node, NodeKind, PortRef, SemanticNodeKind, Severity, TriggerRule, SPEC_VERSION,
};

#[test]
fn validation_error_and_warning_coverage() {
    let expected_error_codes = [
        "E1001", "E1002", "E1003", "E1004", "E1005", "E1007", "E1008", "E1009", "E1010", "E1011",
        "E1020", "E1021", "E1022", "E1023", "E1024", "E1025", "E1027", "E1028", "E1029", "E1030",
        "E1031", "E1032", "E1035", "E1039", "E1040", "E1041", "E1042", "E1043", "E1044", "E1045",
        "E1046", "E1047", "E1048", "E1049", "E1050", "E1051", "E1052", "E1053", "E1054", "E1055",
        "E1056", "E1057", "E1058", "E1059", "E1060", "E1061", "E1062",
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
        inputs: std::collections::BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: std::collections::BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes: vec![
            build_node("a", vec![], "out"),
            build_node("b", vec!["in".to_string()], "out"),
            build_node("c", vec!["in".to_string()], "out"),
        ],
        edges: vec![
            Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
            },
            Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
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
        id: None,
        kind: EdgeKind::Data,
        decision: None,
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
                path_var: None,
            }),
        )]
        .into_iter()
        .collect(),
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn path_variable_reference_validation_rejects_unknown_bindings() {
    let mut graph = base_graph();
    graph.nodes[0].params = bijux_dag_core::ParamValue::Object(
        [(
            "target".to_string(),
            bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: None,
                node_output: None,
                path_var: Some(bijux_dag_core::PathVarRef::Name("unknown_dir".to_string())),
            }),
        )]
        .into_iter()
        .collect(),
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn path_variable_reference_validation_rejects_traversal_suffixes() {
    let mut graph = base_graph();
    graph.nodes[0].params = bijux_dag_core::ParamValue::Object(
        [(
            "target".to_string(),
            bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: None,
                node_output: None,
                path_var: Some(bijux_dag_core::PathVarRef::Binding(
                    bijux_dag_core::PathVarBinding {
                        name: "outputs_dir".to_string(),
                        relative_path: Some("../escape.txt".to_string()),
                    },
                )),
            }),
        )]
        .into_iter()
        .collect(),
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1025"));
}

#[test]
fn container_workdir_validation_rejects_relative_escapes() {
    let mut graph = base_graph();
    graph.nodes[0].kind = NodeKind::Container;
    graph.nodes[0].container = Some(bijux_dag_core::ContainerSpec {
        image: "alpine".to_string(),
        argv: vec!["echo".to_string(), "ok".to_string()],
        env_allowlist: vec![],
        workdir: Some("../escape".to_string()),
        engine: "docker".to_string(),
    });
    graph.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];

    let diags = graph.validate_with_warnings();

    assert!(diags
        .iter()
        .any(|d| { d.code == "E1025" && d.path == "/nodes/source/container/workdir" }));
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
        inputs: std::collections::BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: std::collections::BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes,
        edges: vec![Edge {
            id: None,
            kind: EdgeKind::Data,
            decision: None,
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
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "missing".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            });
            g
        }
        "E1003" => {
            let mut g = base_graph();
            g.nodes[1].inputs = vec!["in".to_string()];
            g.edges.push(Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "source".to_string(), port: "missing".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            });
            g
        }
        "E1004" => {
            let g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: std::collections::BTreeMap::new(),
                nondeterminism_allowed: false,
                subgraphs: std::collections::BTreeMap::new(),
                subgraph_instances: Vec::new(),
                nodes: vec![
                    build_node("a", vec![], "out"),
                    build_node("b", vec!["in".to_string()], "out"),
                ],
                edges: vec![
                    Edge {
                        id: None,
                        kind: EdgeKind::Data,
                        decision: None,
                        from: PortRef { node_id: "a".to_string(), port: "out".to_string() },
                        to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
                    },
                    Edge {
                        id: None,
                        kind: EdgeKind::Data,
                        decision: None,
                        from: PortRef { node_id: "b".to_string(), port: "out".to_string() },
                        to: PortRef { node_id: "a".to_string(), port: "in".to_string() },
                    },
                ],
            };
            g
        }
        "E1005" => {
            let mut g = base_graph();
            g.edges.clear();
            g
        }
        "E1007" => {
            let mut g = base_graph();
            g.nodes.push(build_node("bad node", vec![], "out"));
            g
        }
        "E1008" => {
            let mut g = base_graph();
            g.nodes[0].outputs = vec![FileOutput::new("same".to_string(), "out.txt".to_string())];
            g.nodes[1].outputs = vec![FileOutput::new("same".to_string(), "out.txt".to_string())];
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
                        path_var: None,
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
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec!["in".to_string()],
                outputs: vec![FileOutput::new("out".to_string(), "dep/out.txt".to_string())],
                params: bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                    graph_input: None,
                    node_output: Some(bijux_dag_core::NodeOutputRef {
                        node_id: "ghost".to_string(),
                        output_name: "out".to_string(),
                    }),
                    path_var: None,
                }),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: Default::default(),
                cache: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
                trigger_rule: TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            });
            g
        }
        "E1022" => {
            let mut g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: std::collections::BTreeMap::new(),
                nondeterminism_allowed: false,
                subgraphs: std::collections::BTreeMap::new(),
                subgraph_instances: Vec::new(),
                nodes: vec![
                    build_node("source", vec!["in".to_string()], "out"),
                    build_node("sink", vec!["in".to_string()], "out"),
                ],
                edges: vec![Edge {
                    id: None,
                    kind: EdgeKind::Data,
                    decision: None,
                    from: PortRef { node_id: "source".to_string(), port: "out".to_string() },
                    to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
                }],
            };
            g.nodes[0].params = bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: None,
                node_output: Some(bijux_dag_core::NodeOutputRef {
                    node_id: "sink".to_string(),
                    output_name: "out".to_string(),
                }),
                path_var: None,
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
            g.nodes[0].outputs = vec![FileOutput::new("out".to_string(), "../bad.out".to_string())];
            g
        }
        "E1027" => {
            let mut g = base_graph();
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Env];
            g.nodes[0].env_allowlist = vec!["*".to_string()];
            g
        }
        "E1028" => {
            let mut g = base_graph();
            g.nodes[0].semantic_kind = SemanticNodeKind::Branch;
            g.nodes[0].branch = Some(BranchSpec {
                decisions: vec!["left".to_string()],
                default_decision: Some("left".to_string()),
                decision_output: "missing".to_string(),
            });
            g
        }
        "E1029" => {
            let mut g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: std::collections::BTreeMap::new(),
                nondeterminism_allowed: false,
                subgraphs: std::collections::BTreeMap::new(),
                subgraph_instances: Vec::new(),
                nodes: vec![
                    Node {
                        id: "branch".to_string(),
                        kind: NodeKind::Shell,
                        semantic_kind: SemanticNodeKind::Branch,
                        inputs: vec!["in".to_string()],
                        outputs: vec![FileOutput::new(
                            "decision".to_string(),
                            "branch/decision.txt".to_string(),
                        )],
                        params: bijux_dag_core::ParamValue::default(),
                        container: None,
                        timeout_ms: None,
                        resources: None,
                        tags: vec![],
                        retry: Default::default(),
                        cache: Default::default(),
                        effects: vec![bijux_dag_core::Effect::Filesystem],
                        env_allowlist: vec![],
                        group: None,
                        trigger_rule: TriggerRule::AllSuccess,
                        branch: Some(BranchSpec {
                            decisions: vec!["left".to_string()],
                            default_decision: Some("left".to_string()),
                            decision_output: "decision".to_string(),
                        }),
                        dynamic: None,
                    },
                    build_node("sink", vec!["in".to_string()], "out"),
                ],
                edges: vec![Edge {
                    id: Some("branch-left".to_string()),
                    kind: EdgeKind::Conditional,
                    decision: Some("right".to_string()),
                    from: PortRef { node_id: "branch".to_string(), port: "decision".to_string() },
                    to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
                }],
            };
            g.nodes[1].trigger_rule = TriggerRule::AnySuccess;
            g
        }
        "E1030" => {
            let mut g = Graph {
                spec: SPEC_VERSION.to_string(),
                meta: None,
                inputs: std::collections::BTreeMap::new(),
                nondeterminism_allowed: false,
                subgraphs: std::collections::BTreeMap::new(),
                subgraph_instances: Vec::new(),
                nodes: vec![
                    Node {
                        id: "branch".to_string(),
                        kind: NodeKind::Shell,
                        semantic_kind: SemanticNodeKind::Branch,
                        inputs: vec!["in".to_string()],
                        outputs: vec![FileOutput::new(
                            "decision".to_string(),
                            "branch/decision.txt".to_string(),
                        )],
                        params: bijux_dag_core::ParamValue::default(),
                        container: None,
                        timeout_ms: None,
                        resources: None,
                        tags: vec![],
                        retry: Default::default(),
                        cache: Default::default(),
                        effects: vec![bijux_dag_core::Effect::Filesystem],
                        env_allowlist: vec![],
                        group: None,
                        trigger_rule: TriggerRule::AllSuccess,
                        branch: Some(BranchSpec {
                            decisions: vec!["left".to_string()],
                            default_decision: Some("left".to_string()),
                            decision_output: "decision".to_string(),
                        }),
                        dynamic: None,
                    },
                    build_node("sink", vec!["in".to_string()], "out"),
                ],
                edges: vec![Edge {
                    id: Some("branch-left".to_string()),
                    kind: EdgeKind::Conditional,
                    decision: Some("left".to_string()),
                    from: PortRef { node_id: "branch".to_string(), port: "decision".to_string() },
                    to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
                }],
            };
            g.nodes[1].trigger_rule = TriggerRule::AllSuccess;
            g
        }
        "E1031" => {
            let mut g = base_graph();
            g.inputs.insert(
                "seed".to_string(),
                bijux_dag_core::GraphInputSpec::from_default_value(serde_json::json!(7))
                    .expect("seed spec"),
            );
            g.nodes[1].params = bijux_dag_core::ParamValue::Ref(bijux_dag_core::RefSpec {
                graph_input: Some("seed".to_string()),
                node_output: Some(bijux_dag_core::NodeOutputRef {
                    node_id: "source".to_string(),
                    output_name: "out".to_string(),
                }),
                path_var: None,
            });
            g
        }
        "E1032" => {
            let mut g = base_graph();
            g.nodes[0].cache = CacheBehavior { enabled: false, reason: None };
            g
        }
        "E1035" => {
            let mut g = base_graph();
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Env];
            g
        }
        "E1039" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Python;
            g.nodes[0].params = bijux_dag_core::ParamValue::Literal(serde_json::json!("bad"));
            g
        }
        "E1040" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Python;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "function".to_string(),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("run")),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1041" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Python;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "module".to_string(),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("demo_module")),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1042" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Http;
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network];
            g.nodes[0].params = bijux_dag_core::ParamValue::Literal(serde_json::json!("bad"));
            g
        }
        "E1043" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Http;
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "url".to_string(),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("https://example.test")),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1044" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Http;
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "method".to_string(),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("GET")),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1045" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Http;
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "method".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("GET")),
                    ),
                    (
                        "url".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!(
                            "https://example.test"
                        )),
                    ),
                    (
                        "headers".to_string(),
                        bijux_dag_core::ParamValue::Object(
                            [(
                                "authorization".to_string(),
                                bijux_dag_core::ParamValue::Literal(serde_json::json!(true)),
                            )]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1046" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Http;
            g.nodes[0].effects =
                vec![bijux_dag_core::Effect::Filesystem, bijux_dag_core::Effect::Network];
            g.nodes[0].outputs.clear();
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "method".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("GET")),
                    ),
                    (
                        "url".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!(
                            "https://example.test"
                        )),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1047" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Literal(serde_json::json!("bad"));
            g
        }
        "E1048" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "operation".to_string(),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("rename")),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1049" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("copy")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("../escape.txt")),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1050" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("concatenate")),
                    ),
                    (
                        "sources".to_string(),
                        bijux_dag_core::ParamValue::Array(vec![
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("../bad.txt")),
                        ]),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1051" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].outputs =
                vec![FileOutput::new("chunk".to_string(), "source/chunk-1.txt".to_string())];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("split")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                    ),
                    (
                        "chunk_bytes".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!(0)),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1052" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("checksum")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                    ),
                    (
                        "checksum_algorithm".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("md5")),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1053" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("gzip_compress")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                    ),
                    (
                        "compression_level".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!(12)),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1054" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0].outputs[0].kind = bijux_dag_core::OutputKind::Directory;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("copy")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1055" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::FileTransform;
            g.nodes[0].effects = vec![bijux_dag_core::Effect::Filesystem];
            g.nodes[0]
                .outputs
                .push(FileOutput::new("extra".to_string(), "source/extra.txt".to_string()));
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "operation".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("copy")),
                    ),
                    (
                        "source".to_string(),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("seed/in.txt")),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1056" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Map;
            g.nodes[0].inputs = vec!["in".to_string()];
            g.nodes[0].outputs.clear();
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "argv".to_string(),
                    bijux_dag_core::ParamValue::Array(vec![
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                    ]),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1057" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Map;
            g.nodes[0].inputs = vec!["in".to_string()];
            g.nodes[0].outputs[0].kind = bijux_dag_core::OutputKind::File;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "argv".to_string(),
                    bijux_dag_core::ParamValue::Array(vec![
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                    ]),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1058" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Map;
            g.nodes[0].inputs = vec!["left".to_string(), "right".to_string()];
            g.nodes[0].outputs[0].kind = bijux_dag_core::OutputKind::Directory;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "argv".to_string(),
                    bijux_dag_core::ParamValue::Array(vec![
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                    ]),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1059" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Reduce;
            g.nodes[0].inputs = vec!["mapped".to_string()];
            g.nodes[0]
                .outputs
                .push(FileOutput::new("extra".to_string(), "source/extra.txt".to_string()));
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "argv".to_string(),
                    bijux_dag_core::ParamValue::Array(vec![
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                    ]),
                )]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1060" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Reduce;
            g.nodes[0].inputs = vec!["mapped".to_string()];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "argv".to_string(),
                        bijux_dag_core::ParamValue::Array(vec![
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                        ]),
                    ),
                    (
                        "reduce".to_string(),
                        bijux_dag_core::ParamValue::Object(
                            [(
                                "mode".to_string(),
                                bijux_dag_core::ParamValue::Literal(serde_json::json!(
                                    "best_effort"
                                )),
                            )]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1061" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Reduce;
            g.nodes[0].inputs = vec!["mapped".to_string()];
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [
                    (
                        "argv".to_string(),
                        bijux_dag_core::ParamValue::Array(vec![
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                            bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                        ]),
                    ),
                    (
                        "reduce".to_string(),
                        bijux_dag_core::ParamValue::Object(
                            [(
                                "empty".to_string(),
                                bijux_dag_core::ParamValue::Literal(serde_json::json!("maybe")),
                            )]
                            .into_iter()
                            .collect(),
                        ),
                    ),
                ]
                .into_iter()
                .collect(),
            );
            g
        }
        "E1062" => {
            let mut g = base_graph();
            g.nodes[0].kind = NodeKind::Shell;
            g.nodes[0].semantic_kind = SemanticNodeKind::Reduce;
            g.nodes[0].inputs = vec!["mapped".to_string()];
            g.nodes[0].trigger_rule = TriggerRule::AnySuccess;
            g.nodes[0].params = bijux_dag_core::ParamValue::Object(
                [(
                    "argv".to_string(),
                    bijux_dag_core::ParamValue::Array(vec![
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                        bijux_dag_core::ParamValue::Literal(serde_json::json!("printf ok")),
                    ]),
                )]
                .into_iter()
                .collect(),
            );
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
                inputs: std::collections::BTreeMap::new(),
                nondeterminism_allowed: false,
                subgraphs: std::collections::BTreeMap::new(),
                subgraph_instances: Vec::new(),
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
        semantic_kind: SemanticNodeKind::Task,
        inputs: std::mem::take(&mut inputs),
        outputs: vec![FileOutput::new(name.to_string(), format!("{id}/{name}.txt"))],
        params: bijux_dag_core::ParamValue::default(),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: Default::default(),
        cache: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
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
                id: None,
                kind: EdgeKind::Data,
                decision: None,
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
        inputs: std::collections::BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: std::collections::BTreeMap::new(),
        subgraph_instances: Vec::new(),
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
