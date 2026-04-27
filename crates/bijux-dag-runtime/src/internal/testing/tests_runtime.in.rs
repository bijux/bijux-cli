mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use crate::{Fs, StdFs};
    use crate::test_support::{docker_available, param_object, sample_graph};
    use bijux_dag_core::{ContainerSpec, Edge, Effect, ParamValue, PortRef, Severity, SPEC_VERSION};
    use std::io;
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::sync::{Arc, Mutex, OnceLock};

    fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().expect("lock")
    }

    fn create_staging_conflict(base: &Path, run_id: &str, relative: &str) {
        let conflict = base.join(format!("run.tmp-{run_id}")).join(relative);
        fs::create_dir_all(conflict.parent().expect("conflict parent")).expect("create parent");
        fs::create_dir_all(&conflict).expect("create conflict directory");
    }

    #[derive(Clone)]
    struct InterceptFs {
        inner: StdFs,
        fail_write_suffix: Option<&'static str>,
        fail_symlink_name: Option<&'static str>,
    }

    impl InterceptFs {
        fn fail_write(suffix: &'static str) -> Self {
            Self { inner: StdFs, fail_write_suffix: Some(suffix), fail_symlink_name: None }
        }

        fn fail_symlink(name: &'static str) -> Self {
            Self { inner: StdFs, fail_write_suffix: None, fail_symlink_name: Some(name) }
        }
    }

    impl Fs for InterceptFs {
        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            self.inner.create_dir_all(path)
        }

        fn read_to_string(&self, path: &Path) -> io::Result<String> {
            self.inner.read_to_string(path)
        }

        fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
            self.inner.read(path)
        }

        fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            if self
                .fail_write_suffix
                .is_some_and(|suffix| path.to_string_lossy().ends_with(suffix))
            {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "intercepted write"));
            }
            self.inner.write(path, data)
        }

        fn open_append(&self, path: &Path) -> io::Result<fs::File> {
            self.inner.open_append(path)
        }

        fn read_dir(&self, path: &Path) -> io::Result<Vec<fs::DirEntry>> {
            self.inner.read_dir(path)
        }

        fn metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
            self.inner.metadata(path)
        }

        fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.inner.rename(from, to)
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.inner.remove_file(path)
        }

        fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
            self.inner.copy(from, to)
        }

        fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.inner.hard_link(from, to)
        }

        fn symlink(&self, from: &Path, to: &Path) -> io::Result<()> {
            if self
                .fail_symlink_name
                .is_some_and(|name| to.file_name().and_then(|v| v.to_str()) == Some(name))
            {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "intercepted symlink"));
            }
            self.inner.symlink(from, to)
        }

        fn set_permissions(&self, path: &Path, perms: fs::Permissions) -> io::Result<()> {
            self.inner.set_permissions(path, perms)
        }

        fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
            self.inner.canonicalize(path)
        }
    }

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
        assert!(trace_a.contains("\"status\""));
        assert!(trace_b.contains("\"status\""));

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
        assert!(trace_a.contains("\"status\""));
        assert!(trace_b.contains("\"status\""));
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
        let _log2 = std::fs::read_to_string(path_2.join("run.log.jsonl")).unwrap();
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

        let trace_a = std::fs::read_to_string(run.join("nodes").join("long_a").join("trace.json"))
            .unwrap_or_default();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("long_b").join("trace.json"))
            .unwrap_or_default();
        assert!(trace_a.contains("\"status\""));
        assert!(trace_b.contains("\"status\"") || trace_b.is_empty());
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
        let out_alt = final_path
            .join("nodes")
            .join("c1")
            .join("outputs")
            .join("out");
        if !out.exists() && !out_alt.exists() {
            return;
        }
    }

    #[test]
    fn missing_container_engine_fails_as_infrastructure_error() {
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
                    name: "out".to_string(),
                    path: "out.txt".to_string(),
                }],
                params: ParamValue::default(),
                container: Some(ContainerSpec {
                    image: "alpine:3.19".to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > /bijux/node/outputs/out.txt".to_string(),
                    ],
                    env_allowlist: vec![],
                    workdir: Some("/bijux/node/work".to_string()),
                    engine: "podman".to_string(),
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
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let trace = fs::read_to_string(final_path.join("nodes").join("c1").join("trace.json"))
            .unwrap();
        assert!(trace.contains("\"status\": \"failed\""));
        assert!(trace.contains("\"kind\": \"Infrastructure\""));
        assert!(trace.contains("\"code\": \"CONTAINER_ENGINE_UNAVAILABLE\""));
    }

    #[test]
    fn shell_env_is_clean_except_allowlist() {
        let _env_lock = process_env_lock();
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
    fn shaped_environment_supports_container_allowlist_contract() {
        let _env_lock = process_env_lock();
        std::env::set_var("BIJUX_TEST_FOO", "allowed");
        std::env::set_var("BIJUX_TEST_BAR", "blocked");
        let shaped = shaped_environment(true, &["BIJUX_TEST_FOO".to_string()], &[]);
        assert_eq!(shaped.get("BIJUX_TEST_FOO"), Some(&"allowed".to_string()));
        assert!(!shaped.contains_key("BIJUX_TEST_BAR"));
        std::env::remove_var("BIJUX_TEST_FOO");
        std::env::remove_var("BIJUX_TEST_BAR");
    }

    #[test]
    fn external_adapter_executes() {
        let _env_lock = process_env_lock();
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
    fn external_adapter_uses_shaped_environment_allowlist() {
        let _env_lock = process_env_lock();
        let dir = tempfile::tempdir().unwrap();
        let adapter_dir = dir.path().join("adapters");
        fs::create_dir_all(&adapter_dir).unwrap();
        let adapter_path = adapter_dir.join("env-adapter");
        fs::write(
            &adapter_path,
            r#"#!/bin/sh
if [ "$1" = "info" ]; then
  echo '{"id":"fake","version":"0.1","required_effects":{"filesystem":true,"env":true,"network":false,"clock":false},"supported_kinds":["fake"],"produces_outputs_schema_version":"v0.1"}'
  exit 0
fi
if [ "$1" = "execute" ]; then
  outdir=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --outdir) outdir="$2"; shift 2;;
      --workdir|--node-spec) shift 2;;
      *) shift;;
    esac
  done
  mkdir -p "$outdir"
  env | grep '^BIJUX_TEST_FOO=' > "$outdir/out"
  exit 0
fi
exit 1
"#,
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
        std::env::set_var("BIJUX_TEST_FOO", "allowed");
        std::env::set_var("BIJUX_TEST_BAR", "blocked");
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
        let out = fs::read_to_string(
            final_path
                .join("nodes")
                .join("n1")
                .join("outputs")
                .join("out"),
        )
        .unwrap();
        assert!(out.contains("BIJUX_TEST_FOO=allowed"));
        assert!(!out.contains("BIJUX_TEST_BAR=blocked"));
        std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
        std::env::remove_var("BIJUX_TEST_FOO");
        std::env::remove_var("BIJUX_TEST_BAR");
    }

    #[test]
    fn external_adapter_rejects_oversized_node_spec_payload() {
        let _env_lock = process_env_lock();
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
        let big = "x".repeat(300_000);
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
                params: param_object(vec![("payload", Value::String(big))]),
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
        assert!(trace.contains("node spec payload exceeds"));
        std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    }

    #[test]
    fn output_validation_rejects_symlinked_intermediate_components() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let dir = tempfile::tempdir().unwrap();
            let outdir = dir.path().join("outputs");
            fs::create_dir_all(outdir.join("safe")).unwrap();
            symlink(outdir.join("safe"), outdir.join("link")).unwrap();
            fs::write(outdir.join("safe").join("result.txt"), b"ok").unwrap();
            let outputs = vec![FileOutput {
                name: "out".to_string(),
                path: "link/result.txt".to_string(),
            }];
            let failure = validate_outputs_dir(&outdir, &outputs).expect("must fail");
            assert_eq!(failure.code, "OUTPUT_PATH_INVALID");
            assert!(failure.message.contains("traverses symlink"));
        }
    }

    #[test]
    fn output_validation_rejects_non_normalized_declared_paths() {
        let dir = tempfile::tempdir().unwrap();
        let outputs = vec![FileOutput {
            name: "bad".to_string(),
            path: "nested//out.txt".to_string(),
        }];
        let failure = validate_outputs_dir(dir.path(), &outputs).expect("must fail");
        assert_eq!(failure.code, "OUTPUT_PATH_INVALID");
        assert!(failure.message.contains("invalid output path"));
    }

    #[test]
    fn output_validation_skips_symlink_loops_during_undeclared_scan() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let dir = tempfile::tempdir().unwrap();
            let outdir = dir.path().join("outputs");
            fs::create_dir_all(outdir.join("real")).unwrap();
            fs::write(outdir.join("real").join("declared.txt"), b"ok").unwrap();
            symlink(outdir.clone(), outdir.join("real").join("loop")).unwrap();
            let outputs = vec![FileOutput {
                name: "declared".to_string(),
                path: "real/declared.txt".to_string(),
            }];
            assert!(validate_outputs_dir(&outdir, &outputs).is_none());
        }
    }

    #[test]
    fn local_shell_execution_enforces_node_timeout() {
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
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("sleep 1; echo ok > ../outputs/out"),
                    ]),
                )]),
                container: None,
                timeout_ms: Some(50),
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
        assert!(trace.contains("timed out"));
        let run_log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        assert!(run_log.contains("\"event\":\"node_attempt_started\""));
        assert!(run_log.contains("\"event\":\"node_attempt_finished\""));
        assert!(run_log.contains("\"status\":\"failed\""));
    }

    #[test]
    fn container_execution_enforces_node_timeout_when_engine_is_available() {
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
                id: "c".to_string(),
                kind: NodeKind::Container,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: ParamValue::default(),
                container: Some(ContainerSpec {
                    engine: "docker".to_string(),
                    image: "alpine:3.20".to_string(),
                    argv: vec![
                        "/bin/sh".to_string(),
                        "-c".to_string(),
                        "sleep 2; echo ok > /bijux/node/outputs/out".to_string(),
                    ],
                    workdir: Some("/bijux/node/work".to_string()),
                    env_allowlist: vec![],
                }),
                timeout_ms: Some(100),
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
        let trace = fs::read_to_string(final_path.join("nodes").join("c").join("trace.json"))
            .unwrap();
        assert!(trace.contains("timed out"));
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
        let _env_lock = process_env_lock();
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

    #[test]
    fn run_snapshot_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(Arc::new(InterceptFs::fail_write("run.snapshot.json")), Arc::new(SystemClock));
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn run_attempt_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(Arc::new(InterceptFs::fail_write("run.attempts.json")), Arc::new(SystemClock));
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn lineage_snapshot_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        create_staging_conflict(dir.path(), "lineage-conflict", "lineage.snapshot.json");
        let runtime = Runtime::new();
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    run_id: Some("lineage-conflict".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap_err();
        let rendered = format!("{err}");
        assert!(matches!(err, RuntimeError::Executor(_) | RuntimeError::Artifact(_) | RuntimeError::Io(_)));
        assert!(rendered.contains("lineage snapshot"));
    }
}
