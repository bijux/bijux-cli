mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use crate::test_support::{docker_available, param_object, sample_graph};
    use bijux_dag_core::{ContainerSpec, Edge, Effect, ParamValue, PortRef, Severity, SPEC_VERSION};
    use std::fs;

    #[test]
    fn run_produces_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let diags = sample_graph().validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        assert!(final_path.join("manifest.json").exists());
        assert!(final_path.join("graph.snapshot.json").exists());
        assert!(final_path
            .join("nodes")
            .join("a")
            .join("resolved_params.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("resolved_params.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("trace.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("trace.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("stdout.log")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("outputs")
            .join("index.json")
            .exists());
    }

    #[test]
    fn shell_outputs_index_contains_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(index.contains("out_b"));
    }

    #[test]
    fn artifact_tree_contains_expected_entries() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let expected = vec![
            "manifest.json",
            "provenance.json",
            "graph.snapshot.json",
            "run.log.jsonl",
            "outputs/index.json",
            "nodes/a/trace.json",
            "nodes/a/stdout.log",
            "nodes/a/stderr.log",
            "nodes/a/resolved_params.json",
            "nodes/a/inputs/index.json",
            "nodes/a/outputs/index.json",
            "nodes/a/outputs/out_a",
            "nodes/b/trace.json",
            "nodes/b/stdout.log",
            "nodes/b/stderr.log",
            "nodes/b/resolved_params.json",
            "nodes/b/inputs/index.json",
            "nodes/b/outputs/index.json",
        ];
        for e in expected {
            assert!(final_path.join(e).exists(), "missing {}", e);
        }
    }

    #[test]
    fn failing_node_writes_failure() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].params =
            param_object(vec![("argv", Value::Array(vec![Value::from("false")]))]);
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        if let Err(err) = &result {
            panic!("{:?}", err);
        }
        let final_path = result.unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"failure\""));
    }

    #[test]
    fn jobs_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt1 = RuntimeConfig {
            jobs: 1,
            ..RuntimeConfig::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt1).unwrap();
        let opt2 = RuntimeConfig {
            jobs: 4,
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let snap1 = fs::read_to_string(run1.join("graph.snapshot.json")).unwrap();
        let snap2 = fs::read_to_string(run2.join("graph.snapshot.json")).unwrap();
        assert_eq!(snap1, snap2);
    }

    #[test]
    fn scheduler_order_stable() {
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };
        let order = graph.topo_order().unwrap();
        assert_eq!(order, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn selector_filters_inclusion_and_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let graph = sample_graph();
        let include = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![Selector::Tag("etl".to_string())],
                exclude: vec![],
            },
            ..RuntimeConfig::default()
        };
        let run = runtime.run(&graph, dir.path(), include).unwrap();
        let trace_a = std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("\"status\": \"success\""));

        let dir = tempfile::tempdir().unwrap();
        let exclude = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![],
                exclude: vec![Selector::Tag("etl".to_string())],
            },
            ..RuntimeConfig::default()
        };
        let run = runtime.run(&graph, dir.path(), exclude).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
        let trace_a = std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("\"status\": \"success\""));
    }

    #[test]
    fn replay_run_outputs_are_deterministic() {
        let graph = sample_graph();
        let clock = Arc::new(clock::FixedClock::new(999));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);

        let run1 = tempfile::tempdir().unwrap();
        let path_1 = runtime
            .run(&graph, run1.path(), RuntimeConfig::default())
            .unwrap();
        let out1 = std::fs::read_to_string(path_1.join("outputs").join("index.json")).unwrap();

        let run2 = tempfile::tempdir().unwrap();
        let path_2 = runtime
            .run(&graph, run2.path(), RuntimeConfig::default())
            .unwrap();
        let out2 = std::fs::read_to_string(path_2.join("outputs").join("index.json")).unwrap();

        let log1 = std::fs::read_to_string(path_1.join("run.log.jsonl")).unwrap();
        let log2 = std::fs::read_to_string(path_2.join("run.log.jsonl")).unwrap();
        assert!(log1.contains("run_started"));
        assert_eq!(out1, out2);
    }

    #[test]
    fn run_timeout_skips_after_budget_is_reached() {
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "long_a".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![
                        (
                            "argv",
                            serde_json::json!(
                                ["/bin/sh", "-c", "sleep 0.05; echo done > ../outputs/out_a"]
                            ),
                        ),
                    ]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "long_b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![
                        (
                            "argv",
                            serde_json::json!(
                                ["/bin/sh", "-c", "echo skipped > ../outputs/out_b"]
                            ),
                        ),
                    ]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let run = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    run_timeout_ms: Some(10),
                    jobs: 1,
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let trace_a = std::fs::read_to_string(run.join("nodes").join("long_a").join("trace.json")).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("long_b").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"success\""));
        assert!(trace_b.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("run_timeout"));
    }

    #[test]
    fn cache_corruption_forces_recompute() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        // corrupt cache by deleting an output file
        let entries: Vec<_> = fs::read_dir(cache_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let entry = entries[0].path();
        let index_path = entry.join("outputs").join("index.json");
        if let Ok(data) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<OutputsIndex>(&data) {
                if let Some(file) = index.files.first() {
                    let out_file = entry.join("outputs").join(&file.path);
                    if out_file.exists() {
                        fs::remove_file(out_file).unwrap();
                    }
                }
            }
        }

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();

        let trace_a = fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b = fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap();
        let has_corrupt =
            trace_a.contains("\"corrupt_detected\"") || trace_b.contains("\"corrupt_detected\"");
        assert!(has_corrupt);

        // ensure outputs still exist
        assert!(run1
            .join("nodes")
            .join("b")
            .join("outputs")
            .join("index.json")
            .exists());
    }

    #[test]
    fn remote_cache_hit() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        let trace_b: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap(),
        )
        .unwrap();
        let src_a = trace_a
            .get("cache_proof")
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str());
        let src_b = trace_b
            .get("cache_proof")
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str());
        assert!(src_a == Some("remote") || src_b == Some("remote"));
    }

    #[test]
    fn remote_cache_corruption_reexecutes() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let entries: Vec<_> = fs::read_dir(remote_cache.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let entry = entries[0].path();
        let index_path = entry.join("outputs").join("index.json");
        if let Ok(data) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<OutputsIndex>(&data) {
                if let Some(file) = index.files.first() {
                    let out_file = entry.join("outputs").join(&file.path);
                    if out_file.exists() {
                        fs::remove_file(out_file).unwrap();
                    }
                }
            }
        }

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        let trace_b: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap(),
        )
        .unwrap();
        let bad_a = trace_a
            .get("cache_proof")
            .and_then(|v| v.get("corrupt_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let bad_b = trace_b
            .get("cache_proof")
            .and_then(|v| v.get("corrupt_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(bad_a || bad_b);
    }

    #[test]
    fn fixed_clock_produces_stable_event_ts() {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(FixedClock::new(123));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        for line in log.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(v.get("ts").and_then(|v| v.as_u64()), Some(123));
        }
    }

    #[test]
    fn downstream_reads_upstream_file() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from("hello"))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("cat ../inputs/a/in > ../outputs/out_b"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "out_a".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        };
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        if let Err(err) = &result {
            panic!("{:?}", err);
        }
        let final_path = result.unwrap();
        let out = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("out_b"),
        )
        .unwrap();
        assert!(out.contains("hello"));
        let inputs_index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("inputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(inputs_index.contains("a/in"));
    }

    #[test]
    fn file_wiring_only_materializes_bound_output() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![
                        FileOutput {
                            name: "a".to_string(),
                            path: "a.txt".to_string(),
                        },
                        FileOutput {
                            name: "b".to_string(),
                            path: "b.txt".to_string(),
                        },
                    ],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("echo a > ../outputs/a.txt; echo b > ../outputs/b.txt"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("if [ ! -f ../inputs/a/in ]; then exit 1; fi; if [ -e ../inputs/a/b ]; then exit 1; fi; cat ../inputs/a/in > ../outputs/out_b"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "a".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        };
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("out_b"),
        )
        .unwrap();
        assert!(out.contains("a"));
        let inputs_index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("inputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(inputs_index.contains("a/in"));
        assert!(!inputs_index.contains("a/b"));
    }

    #[test]
    fn retry_succeeds_on_second_attempt() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].retry = bijux_dag_core::RetryPolicy {
            max_attempts: 1,
            backoff_ms: 0,
        };
        graph.nodes[1].params = param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from(
                    "if [ ! -f marker ]; then touch marker; exit 1; fi; echo ok > ../outputs/out_b",
                ),
            ]),
        )]);
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"attempt\": 2"));
        assert!(trace.contains("\"status\""));
    }

    #[test]
    fn cpu_budget_schedules_deterministically() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "c".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_c".to_string(),
                        path: "out_c".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(3))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let opt = RuntimeConfig {
            jobs: 3,
            cpu_budget: Some(2),
            ..RuntimeConfig::default()
        };
        let final_path = runtime.run(&graph, dir.path(), opt).unwrap();
        let log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        let mut scheduled = Vec::new();
        for line in log.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            if v.get("event") == Some(&Value::String("node_scheduled".to_string())) {
                if let Some(id) = v.get("node_id").and_then(|v| v.as_str()) {
                    scheduled.push(id.to_string());
                }
            }
        }
        assert_eq!(scheduled, vec!["a", "b", "c"]);
    }

    #[test]
    fn container_node_writes_output() {
        if !docker_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "c1".to_string(),
                kind: NodeKind::Container,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out_c".to_string(),
                    path: "out_c".to_string(),
                }],
                params: ParamValue::default(),
                container: Some(ContainerSpec {
                    image: "alpine:3.19".to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > /bijux/node/outputs/out_c".to_string(),
                    ],
                    env_allowlist: vec![],
                    workdir: Some("/bijux/node/work".to_string()),
                    engine: "docker".to_string(),
                }),
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = final_path
            .join("nodes")
            .join("c1")
            .join("outputs")
            .join("out_c");
        assert!(out.exists());
    }

    #[test]
    fn shell_env_is_clean_except_allowlist() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("BIJUX_TEST_FOO", "allowed");
        std::env::set_var("BIJUX_TEST_BAR", "blocked");
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "env".to_string(),
                kind: NodeKind::Shell,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("env"),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem, Effect::Env],
                env_allowlist: vec!["BIJUX_TEST_FOO".to_string()],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let stdout =
            fs::read_to_string(final_path.join("nodes").join("env").join("stdout.log")).unwrap();
        assert!(stdout.contains("BIJUX_TEST_FOO=allowed"));
        assert!(!stdout.contains("BIJUX_TEST_BAR=blocked"));
    }

    #[test]
    fn external_adapter_executes() {
        let dir = tempfile::tempdir().unwrap();
        let adapter_dir = dir.path().join("adapters");
        fs::create_dir_all(&adapter_dir).unwrap();
        let adapter_path = adapter_dir.join("fake-adapter");
        fs::write(
            &adapter_path,
            include_str!("../../../tests/bin/fake_adapter.sh"),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&adapter_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&adapter_path, perms).unwrap();
        }
        std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);

        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("fake".to_string()),
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };

        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = final_path
            .join("nodes")
            .join("n1")
            .join("outputs")
            .join("out");
        assert!(out.exists());
        std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    }

    #[test]
    fn undeclared_output_file_fails_execution() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::Shell,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "declared".to_string(),
                    path: "declared.txt".to_string(),
                }],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("echo ok > ../outputs/declared.txt && echo bad > ../outputs/extra.txt"),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let trace = fs::read_to_string(final_path.join("nodes").join("n1").join("trace.json"))
            .unwrap();
        assert!(trace.contains("\"OUTPUT_UNDECLARED\""));
    }

    #[test]
    fn adapter_metadata_present_for_run_and_replay() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let original = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let replay = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let trace_a = fs::read_to_string(original.join("nodes").join("a").join("trace.json"))
            .unwrap();
        let trace_b = fs::read_to_string(replay.join("nodes").join("a").join("trace.json"))
            .unwrap();
        assert!(trace_a.contains("\"adapter_id\""));
        assert!(trace_b.contains("\"adapter_id\""));
    }

    #[test]
    fn external_adapter_path_must_be_file() {
        let dir = tempfile::tempdir().unwrap();
        let adapter_dir = dir.path().join("adapters");
        fs::create_dir_all(&adapter_dir).unwrap();
        std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);
        let adapters = crate::external_adapter::discover_external_adapters().unwrap();
        assert!(adapters.is_empty());
        std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    }

    #[test]
    fn hermetic_mode_materializes_only_declared_inputs() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let mut graph = sample_graph();
        graph.nodes[1].inputs = vec!["in".to_string()];
        let options = RuntimeConfig {
            policy: PolicyConfig {
                deny_network: true,
                deny_env: true,
                deny_clock: true,
                clean_env: true,
            },
            ..RuntimeConfig::default()
        };
        let final_path = runtime.run(&graph, dir.path(), options).unwrap();
        let inputs_index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("inputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(inputs_index.contains("\"path\": \"a/in\""));
        assert!(!inputs_index.contains("\"path\": \"a/undeclared\""));
    }

    #[test]
    fn missing_adapter_registration_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("missing-kind".to_string()),
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn adapter_version_changes_do_not_change_graph_fingerprint_contract() {
        let graph = sample_graph();
        let before = graph.graph_fingerprint().unwrap();
        let adapters = registered_adapters();
        assert!(!adapters.is_empty());
        let after = graph.graph_fingerprint().unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn cache_modes_execute_in_contract_matrix() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        for mode in [CacheMode::Off, CacheMode::Read, CacheMode::ReadWrite] {
            let options = RuntimeConfig {
                cache_mode: mode,
                ..RuntimeConfig::default()
            };
            let run_dir = runtime.run(&sample_graph(), dir.path(), options).unwrap();
            assert!(run_dir.join("manifest.json").exists());
        }
    }

    #[test]
    fn run_id_collision_is_deterministic_error() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let options = RuntimeConfig {
            run_id: Some("fixed-run-id".to_string()),
            ..RuntimeConfig::default()
        };
        let first = runtime.run(&sample_graph(), dir.path(), options.clone());
        assert!(first.is_ok());
        let second = runtime.run(&sample_graph(), dir.path(), options);
        assert!(second.is_err());
    }

    #[test]
    fn latest_symlink_updates_do_not_mutate_previous_run() {
        let dir = tempfile::tempdir().unwrap();
        let latest = dir.path().join("latest");
        let runtime = Runtime::new();
        let first = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    latest_symlink: Some(latest.clone()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let first_manifest = fs::read_to_string(first.join("manifest.json")).unwrap();

        let second = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    latest_symlink: Some(latest.clone()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let first_manifest_after = fs::read_to_string(first.join("manifest.json")).unwrap();
        let second_manifest = fs::read_to_string(second.join("manifest.json")).unwrap();
        assert_eq!(first_manifest, first_manifest_after);
        assert_ne!(first_manifest_after, second_manifest);
    }
}
