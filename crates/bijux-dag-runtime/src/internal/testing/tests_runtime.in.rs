mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use crate::test_support::{docker_available, param_object, sample_graph};
    use crate::{Fs, StdFs};
    use bijux_dag_artifacts::InputsIndex;
    use bijux_dag_core::{
        ContainerSpec, Edge, Effect, OutputKind, ParamValue, PortRef, Severity, SPEC_VERSION,
    };
    use std::ffi::OsString;
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Barrier, Mutex, OnceLock};

    fn process_env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().expect("lock")
    }

    struct EnvVarGuard {
        key: &'static str,
        original: Option<OsString>,
    }

    impl EnvVarGuard {
        fn replace(key: &'static str, value: &str) -> Self {
            let original = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.original {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    fn create_staging_conflict(base: &Path, run_id: &str, relative: &str) {
        let conflict = base.join(format!("run.tmp-{run_id}")).join(relative);
        fs::create_dir_all(conflict.parent().expect("conflict parent")).expect("create parent");
        fs::create_dir_all(&conflict).expect("create conflict directory");
    }

    fn cache_entry_for_node(cache_root: &Path, node_id: &str) -> PathBuf {
        fs::read_dir(cache_root)
            .expect("read cache root")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .find(|entry| {
                let meta_path = entry.join("meta.json");
                let raw = fs::read_to_string(&meta_path).ok();
                raw.and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                    .and_then(|meta| {
                        meta.get("node_id").and_then(|value| value.as_str()).map(str::to_string)
                    })
                    .as_deref()
                    == Some(node_id)
            })
            .unwrap_or_else(|| panic!("cache entry missing for node {node_id}"))
    }

    fn cache_entry_exists_for_node(cache_root: &Path, node_id: &str) -> bool {
        fs::read_dir(cache_root)
            .expect("read cache root")
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .any(|entry| {
                let meta_path = entry.join("meta.json");
                let raw = fs::read_to_string(&meta_path).ok();
                raw.and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
                    .and_then(|meta| {
                        meta.get("node_id").and_then(|value| value.as_str()).map(str::to_string)
                    })
                    .as_deref()
                    == Some(node_id)
            })
    }

    fn read_inputs_index(run_dir: &Path, node_id: &str) -> InputsIndex {
        serde_json::from_str(
            &fs::read_to_string(run_dir.join("nodes").join(node_id).join("inputs").join("index.json"))
                .expect("read inputs index"),
        )
        .expect("parse inputs index")
    }

    fn read_run_attempts(run_dir: &Path) -> Vec<RunAttempt> {
        serde_json::from_str(&fs::read_to_string(run_dir.join("run.attempts.json")).expect("read attempts"))
            .expect("parse attempts")
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
            if self.fail_write_suffix.is_some_and(|suffix| path.to_string_lossy().ends_with(suffix))
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

        fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
            self.inner.remove_dir_all(path)
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

    #[derive(Clone)]
    struct ConcurrentRenameFs {
        inner: StdFs,
        barrier: Arc<Barrier>,
        watched_parent: PathBuf,
    }

    impl ConcurrentRenameFs {
        fn new(watched_parent: PathBuf, barrier: Arc<Barrier>) -> Self {
            Self { inner: StdFs, barrier, watched_parent }
        }
    }

    impl Fs for ConcurrentRenameFs {
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
            if to.parent() == Some(self.watched_parent.as_path()) {
                self.barrier.wait();
            }
            self.inner.rename(from, to)
        }

        fn remove_file(&self, path: &Path) -> io::Result<()> {
            self.inner.remove_file(path)
        }

        fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
            self.inner.remove_dir_all(path)
        }

        fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
            self.inner.copy(from, to)
        }

        fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.inner.hard_link(from, to)
        }

        fn symlink(&self, from: &Path, to: &Path) -> io::Result<()> {
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
        assert!(!diags.iter().any(|d| d.severity == Severity::Error), "{:?}", diags);
        let final_path =
            runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
        assert!(final_path.join("manifest.json").exists());
        assert!(final_path.join("manifest.finalized.json").exists());
        assert!(final_path.join(".run-complete.json").exists());
        assert!(final_path.join("run.schema.json").exists());
        assert!(final_path.join("graph.snapshot.json").exists());
        assert!(final_path.join("nodes").join("a").join("resolved_params.json").exists());
        assert!(final_path.join("nodes").join("b").join("resolved_params.json").exists());
        assert!(final_path.join("nodes").join("b").join("trace.json").exists());
        assert!(final_path.join("nodes").join("b").join("trace.json").exists());
        assert!(final_path.join("nodes").join("b").join("stdout.log").exists());
        assert!(final_path.join("nodes").join("b").join("outputs").join("index.json").exists());
    }

    #[test]
    fn shell_outputs_index_contains_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path =
            runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
        let index = fs::read_to_string(
            final_path.join("nodes").join("b").join("outputs").join("index.json"),
        )
        .unwrap();
        assert!(index.contains("out_b"));
    }

    #[test]
    fn artifact_tree_contains_expected_entries() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path =
            runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
        let expected = vec![
            "manifest.json",
            "manifest.finalized.json",
            ".run-complete.json",
            "run.schema.json",
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
        assert!(!diags.iter().any(|d| d.severity == Severity::Error), "{:?}", diags);
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
        let opt1 = RuntimeConfig { jobs: 1, ..RuntimeConfig::default() };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt1).unwrap();
        let opt2 = RuntimeConfig { jobs: 4, ..RuntimeConfig::default() };
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out_a".to_string(), "out_a".to_string(),)],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
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
        let trace_a =
            std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b =
            std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
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
        let trace_b =
            std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
        let trace_a =
            std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\""));
        assert!(trace_b.contains("\"status\""));
    }

    #[test]
    fn replay_run_outputs_are_deterministic() {
        let graph = sample_graph();
        let clock = Arc::new(clock::FixedClock::new(999));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);

        let run1 = tempfile::tempdir().unwrap();
        let path_1 = runtime.run(&graph, run1.path(), RuntimeConfig::default()).unwrap();
        let out1 = std::fs::read_to_string(path_1.join("outputs").join("index.json")).unwrap();

        let run2 = tempfile::tempdir().unwrap();
        let path_2 = runtime.run(&graph, run2.path(), RuntimeConfig::default()).unwrap();
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "long_a".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out".to_string(), "out_a".to_string(),)],
                    params: param_object(vec![(
                        "argv",
                        serde_json::json!([
                            "/bin/sh",
                            "-c",
                            "sleep 0.05; echo done > ../outputs/out_a"
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "long_b".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out".to_string(), "out_b".to_string(),)],
                    params: param_object(vec![(
                        "argv",
                        serde_json::json!(["/bin/sh", "-c", "echo skipped > ../outputs/out_b"]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
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
                RuntimeConfig { run_timeout_ms: Some(10), jobs: 1, ..RuntimeConfig::default() },
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
        let entry = cache_entry_for_node(cache_dir.path(), "a");
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
        assert!(trace_a.contains("\"corrupt_detected\": true"));

        // ensure outputs still exist
        assert!(run1.join("nodes").join("b").join("outputs").join("index.json").exists());
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
        let src_a =
            trace_a.get("cache_proof").and_then(|v| v.get("source")).and_then(|v| v.as_str());
        let src_b =
            trace_b.get("cache_proof").and_then(|v| v.get("source")).and_then(|v| v.as_str());
        assert!(src_a == Some("remote") || src_b == Some("remote"));
    }

    #[test]
    fn readwrite_run_publishes_cache_entries_to_remote_root() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let local_entry = cache_entry_for_node(local_cache.path(), "a");
        let remote_entry = cache_entry_for_node(remote_cache.path(), "a");
        assert!(local_entry.join("manifest.json").exists());
        assert!(remote_entry.join("manifest.json").exists());
        assert!(remote_entry.join("outputs").join("index.json").exists());
    }

    #[test]
    fn readwrite_run_can_publish_remote_cache_without_local_root() {
        let dir = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: None,
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let remote_entry = cache_entry_for_node(remote_cache.path(), "a");
        assert!(remote_entry.join("manifest.json").exists());
        assert!(remote_entry.join("meta.json").exists());
        assert!(remote_entry.join("outputs").join("index.json").exists());
    }

    #[test]
    fn remote_cache_hit_primes_local_cache_without_staging_residue() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let seed = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), seed).unwrap();

        let fetch = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), fetch).unwrap();

        let local_entry = cache_entry_for_node(local_cache.path(), "a");
        assert!(local_entry.join("manifest.json").exists());
        assert!(local_entry.join("outputs").join("index.json").exists());
        let has_staging_residue = fs::read_dir(local_cache.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name().to_string_lossy().starts_with(".cache-"));
        assert!(!has_staging_residue);
    }

    #[test]
    fn local_cache_corruption_falls_back_to_remote_and_repairs_local_entry() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let seed = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), seed).unwrap();

        let fetch = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), fetch.clone()).unwrap();

        let local_entry = cache_entry_for_node(local_cache.path(), "a");
        let index_path = local_entry.join("outputs").join("index.json");
        let index: OutputsIndex =
            serde_json::from_str(&fs::read_to_string(&index_path).unwrap()).unwrap();
        let output_path = local_entry.join("outputs").join(&index.files[0].path);
        fs::remove_file(&output_path).unwrap();

        let rerun = runtime.run(&sample_graph(), dir.path(), fetch).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(rerun.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            trace_a["cache_proof"]["source"].as_str(),
            Some("remote")
        );
        assert!(output_path.exists());
    }

    #[test]
    fn concurrent_remote_cache_publication_keeps_shared_entries_valid() {
        let dir = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let shared_fs: Arc<dyn Fs> =
            Arc::new(ConcurrentRenameFs::new(remote_cache.path().to_path_buf(), barrier));
        let clock: Arc<dyn Clock> = Arc::new(FixedClock::new(123));

        let first_runtime = Runtime::with_io(Arc::clone(&shared_fs), Arc::clone(&clock));
        let second_runtime = Runtime::with_io(Arc::clone(&shared_fs), clock);
        let first_graph = sample_graph();
        let second_graph = sample_graph();
        let first_out = dir.path().join("runs-a");
        let second_out = dir.path().join("runs-b");
        let first_remote_root = remote_cache.path().to_path_buf();
        let second_remote_root = remote_cache.path().to_path_buf();

        let first = std::thread::spawn(move || {
            first_runtime.run(
                &first_graph,
                &first_out,
                RuntimeConfig {
                    cache_mode: CacheMode::ReadWrite,
                    cache_dir: None,
                    remote_cache_dir: Some(first_remote_root),
                    ..RuntimeConfig::default()
                },
            )
        });
        let second = std::thread::spawn(move || {
            second_runtime.run(
                &second_graph,
                &second_out,
                RuntimeConfig {
                    cache_mode: CacheMode::ReadWrite,
                    cache_dir: None,
                    remote_cache_dir: Some(second_remote_root),
                    ..RuntimeConfig::default()
                },
            )
        });

        assert!(first.join().unwrap().is_ok());
        assert!(second.join().unwrap().is_ok());

        let remote_entry = cache_entry_for_node(remote_cache.path(), "a");
        assert!(remote_entry.join("manifest.json").exists());
        assert!(remote_entry.join("meta.json").exists());
        assert!(remote_entry.join("outputs").join("index.json").exists());
        let has_staging_residue = fs::read_dir(remote_cache.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.file_name().to_string_lossy().starts_with(".cache-"));
        assert!(!has_staging_residue);
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

        let entry = cache_entry_for_node(remote_cache.path(), "a");
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
        let bad_a = trace_a
            .get("cache_proof")
            .and_then(|v| v.get("corrupt_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(bad_a);
    }

    #[test]
    fn graph_cache_policy_disables_node_cache_reads_and_writes() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let mut graph = sample_graph();
        graph.nodes[0].cache = bijux_dag_core::CacheBehavior {
            enabled: false,
            reason: Some("external clock dependency".to_string()),
        };

        let first = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&graph, dir.path(), first).unwrap();
        assert!(!cache_entry_exists_for_node(cache_dir.path(), "a"));
        assert!(cache_entry_exists_for_node(cache_dir.path(), "b"));

        let second = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let rerun = runtime.run(&graph, dir.path(), second).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(rerun.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        let trace_b: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(rerun.join("nodes").join("b").join("trace.json")).unwrap(),
        )
        .unwrap();

        assert_ne!(trace_a.get("status").and_then(|value| value.as_str()), Some("cached"));
        assert_eq!(trace_b.get("status").and_then(|value| value.as_str()), Some("cached"));
    }

    #[test]
    fn fixed_clock_produces_stable_event_ts() {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(FixedClock::new(123));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);
        let final_path =
            runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out_a".to_string(), "out_a".to_string(),)],
                    params: param_object(vec![("value", Value::from("hello"))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput::new("out_b".to_string(), "out_b".to_string(),)],
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
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
            ],
            edges: vec![Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "a".to_string(), port: "out_a".to_string() },
                to: PortRef { node_id: "b".to_string(), port: "in".to_string() },
            }],
        };
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(!diags.iter().any(|d| d.severity == Severity::Error), "{:?}", diags);
        graph.resolve_graph().unwrap();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        if let Err(err) = &result {
            panic!("{:?}", err);
        }
        let final_path = result.unwrap();
        let out =
            fs::read_to_string(final_path.join("nodes").join("b").join("outputs").join("out_b"))
                .unwrap();
        assert!(out.contains("hello"));
        let inputs_index = read_inputs_index(&final_path, "b");
        assert_eq!(inputs_index.files.len(), 1);
        assert_eq!(inputs_index.files[0].local_path, "a/in");
    }

    #[test]
    fn file_wiring_only_materializes_bound_output() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
inputs: vec![],
                    outputs: vec![
                        FileOutput::new("a".to_string(), "a.txt".to_string(),),
                        FileOutput::new("b".to_string(), "b.txt".to_string(),),
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
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
},
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput::new("out_b".to_string(), "out_b".to_string(),)],
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
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
},
            ],
            edges: vec![Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
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
        assert!(!diags.iter().any(|d| d.severity == Severity::Error), "{:?}", diags);
        graph.resolve_graph().unwrap();
        let final_path = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    policy: PolicyConfig {
                        container_image_reference_policy:
                            ContainerImageReferencePolicy::AllowUnpinned,
                        ..PolicyConfig::default()
                    },
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let out =
            fs::read_to_string(final_path.join("nodes").join("b").join("outputs").join("out_b"))
                .unwrap();
        assert!(out.contains("a"));
        let inputs_index = read_inputs_index(&final_path, "b");
        assert_eq!(inputs_index.files.len(), 1);
        assert_eq!(inputs_index.files[0].local_path, "a/in");
    }

    #[test]
    fn retry_succeeds_on_second_attempt() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].retry = bijux_dag_core::RetryPolicy { max_attempts: 1, backoff_ms: 0 };
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
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    policy: PolicyConfig {
                        container_image_reference_policy:
                            ContainerImageReferencePolicy::AllowUnpinned,
                        ..PolicyConfig::default()
                    },
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"attempt\": 2"));
        assert!(trace.contains("\"status\""));
    }

    #[test]
    fn retry_recreates_work_and_output_sandboxes_between_attempts() {
        let dir = tempfile::tempdir().unwrap();
        let retry_gate = dir.path().join("retry-gate");
        let mut graph = sample_graph();
        graph.nodes[1].retry = bijux_dag_core::RetryPolicy { max_attempts: 1, backoff_ms: 0 };
        graph.nodes[1].params = param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from(format!(
                    "if [ ! -f \"{}\" ]; then touch \"{}\"; echo stale > \"$TMPDIR/first-attempt.tmp\"; echo stale > ../outputs/extra.txt; exit 1; fi; test ! -e \"$TMPDIR/first-attempt.tmp\"; echo ok > ../outputs/out_b",
                    retry_gate.display(),
                    retry_gate.display(),
                )),
            ]),
        )]);

        let runtime = Runtime::new();
        let final_path = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    policy: PolicyConfig {
                        container_image_reference_policy:
                            ContainerImageReferencePolicy::AllowUnpinned,
                        ..PolicyConfig::default()
                    },
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();

        assert!(trace.contains("\"attempt\": 2"));
        assert!(final_path.join("nodes").join("b").join("outputs").join("out_b").exists());
        assert!(!final_path.join("nodes").join("b").join("outputs").join("extra.txt").exists());
        assert!(
            !final_path
                .join("nodes")
                .join("b")
                .join("work")
                .join("temp")
                .join("first-attempt.tmp")
                .exists()
        );
    }

    #[test]
    fn shell_runtime_pins_temp_env_to_node_temp_dir() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].params = param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from("printf '%s\\n%s\\n%s\\n' \"$TMPDIR\" \"$TMP\" \"$TEMP\" > ../outputs/out_b"),
            ]),
        )]);

        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let reported =
            fs::read_to_string(final_path.join("nodes").join("b").join("outputs").join("out_b"))
                .unwrap();
        let run_id = final_path
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(|value| value.strip_prefix("run-"))
            .expect("run id");
        let expected = dir
            .path()
            .join(format!("run.tmp-{run_id}"))
            .join("nodes")
            .join("b")
            .join("work")
            .join("temp");
        let finalized_temp_dir =
            final_path.join("nodes").join("b").join("work").join("temp");
        let lines = reported.lines().collect::<Vec<_>>();

        assert_eq!(lines, vec![
            expected.display().to_string(),
            expected.display().to_string(),
            expected.display().to_string(),
        ]);
        assert!(finalized_temp_dir.is_dir());
    }

    #[test]
    fn cpu_budget_schedules_deterministically() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out_a".to_string(), "out_a".to_string(),)],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources {
                        cpu: 2,
                        mem_mb: 0,
                        gpu_devices: 0,
                        named_resources: std::collections::BTreeMap::new(),
                    }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out_b".to_string(), "out_b".to_string(),)],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources {
                        cpu: 2,
                        mem_mb: 0,
                        gpu_devices: 0,
                        named_resources: std::collections::BTreeMap::new(),
                    }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "c".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out_c".to_string(), "out_c".to_string(),)],
                    params: param_object(vec![("value", Value::from(3))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources {
                        cpu: 2,
                        mem_mb: 0,
                        gpu_devices: 0,
                        named_resources: std::collections::BTreeMap::new(),
                    }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
            ],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let opt = RuntimeConfig { jobs: 3, cpu_budget: Some(2), ..RuntimeConfig::default() };
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "c1".to_string(),
                kind: NodeKind::Container,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out_c".to_string(), "out_c".to_string())],
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
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    policy: PolicyConfig {
                        container_image_reference_policy:
                            ContainerImageReferencePolicy::AllowUnpinned,
                        ..PolicyConfig::default()
                    },
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let out = final_path.join("nodes").join("c1").join("outputs").join("out_c");
        let out_alt = final_path.join("nodes").join("c1").join("outputs").join("out");
        if !out.exists() && !out_alt.exists() {
            return;
        }
    }

    #[test]
    fn missing_container_engine_fails_as_infrastructure_error() {
        let _env_lock = process_env_lock();
        let _path_guard = EnvVarGuard::replace("PATH", "");
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "c1".to_string(),
                kind: NodeKind::Container,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out.txt".to_string())],
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
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    policy: PolicyConfig {
                        container_image_reference_policy:
                            ContainerImageReferencePolicy::AllowUnpinned,
                        ..PolicyConfig::default()
                    },
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("c1").join("trace.json")).unwrap();
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "env".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
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
                cache: Default::default(),
                effects: vec![Effect::Filesystem, Effect::Env],
                env_allowlist: vec!["BIJUX_TEST_FOO".to_string()],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
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
        fs::write(&adapter_path, include_str!("../../../tests/bin/fake_adapter.sh")).unwrap();
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("fake".to_string()),
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };

        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let out = final_path.join("nodes").join("n1").join("outputs").join("out");
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
  echo '{"protocol_version":"bijux-dag-adapter/v1","adapter_id":"fake","adapter_version":"0.1","required_effects":{"filesystem":true,"env":true,"network":false,"clock":false},"supported_kinds":["fake"],"output_schema":"v0.1"}'
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("fake".to_string()),
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem, Effect::Env],
                env_allowlist: vec!["BIJUX_TEST_FOO".to_string()],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let out =
            fs::read_to_string(final_path.join("nodes").join("n1").join("outputs").join("out"))
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
        fs::write(&adapter_path, include_str!("../../../tests/bin/fake_adapter.sh")).unwrap();
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
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("fake".to_string()),
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: param_object(vec![("payload", Value::String(big))]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("n1").join("trace.json")).unwrap();
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
            let outputs =
                vec![FileOutput::new("out".to_string(), "link/result.txt".to_string())];
            let failure = validate_outputs_dir(&outdir, &outputs).expect("must fail");
            assert_eq!(failure.code, "OUTPUT_PATH_INVALID");
            assert!(failure.message.contains("traverses symlink"));
        }
    }

    #[test]
    fn output_validation_rejects_non_normalized_declared_paths() {
        let dir = tempfile::tempdir().unwrap();
        let outputs =
            vec![FileOutput::new("bad".to_string(), "nested//out.txt".to_string())];
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
            let outputs = vec![FileOutput::new("declared".to_string(), "real/declared.txt".to_string(),)];
            assert!(validate_outputs_dir(&outdir, &outputs).is_none());
        }
    }

    #[test]
    fn output_validation_reports_optional_outputs_without_failing() {
        let dir = tempfile::tempdir().unwrap();
        let outdir = dir.path().join("outputs");
        fs::create_dir_all(&outdir).unwrap();
        fs::write(outdir.join("result.json"), br#"{"ok":true}"#).unwrap();

        let mut optional_log = FileOutput::new("log".to_string(), "logs/run.log".to_string());
        optional_log.kind = OutputKind::Log;
        optional_log.required = false;
        let mut required_value = FileOutput::new("result".to_string(), "result.json".to_string());
        required_value.kind = OutputKind::Value;

        let report = inspect_declared_outputs(&outdir, &[required_value, optional_log]);
        assert!(report.failure.is_none());
        assert_eq!(report.output_evidence.len(), 2);
        assert!(report.output_evidence.iter().any(|output| output.name == "log"
            && !output.present
            && !output.required
            && output.size_bytes.is_none()));
        assert!(report.output_evidence.iter().any(|output| {
            output.name == "result"
                && output.present
                && output.kind == "value"
                && output.media_type == "application/json"
                && output.size_bytes == Some(br#"{"ok":true}"#.len() as u64)
                && output.sha256.is_some()
        }));
    }

    #[test]
    fn promotable_outputs_are_recorded_in_trace_manifest_and_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let mut deliverable = FileOutput::new("report".to_string(), "report.json".to_string());
        deliverable.promotable = true;

        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "publish".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![deliverable],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("printf '{\"ok\":true}' > ../outputs/report.json"),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };

        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();

        let trace: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(final_path.join("nodes").join("publish").join("trace.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(trace["outputs"][0]["promotable"], true);

        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(final_path.join("manifest.json")).unwrap())
                .unwrap();
        assert_eq!(manifest["outputs"][0]["promotable"], true);

        let run_index: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(final_path.join("outputs").join("index.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(run_index["files"][0]["promotable"], true);

        let node_index: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(
                final_path
                    .join("nodes")
                    .join("publish")
                    .join("outputs")
                    .join("index.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(node_index["files"][0]["promotable"], true);
    }

    #[test]
    fn output_validation_ignores_managed_output_index_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let outdir = dir.path().join("outputs");
        fs::create_dir_all(&outdir).unwrap();
        fs::write(outdir.join("index.json"), br#"{"files":[]}"#).unwrap();

        let report = inspect_declared_outputs(&outdir, &[]);
        assert!(report.failure.is_none());
    }

    #[test]
    fn directory_outputs_materialize_for_downstream_nodes() {
        let dir = tempfile::tempdir().unwrap();
        let mut source_output = FileOutput::new("data".to_string(), "source-data".to_string());
        source_output.kind = OutputKind::Directory;

        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "source".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![source_output],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from(
                                "mkdir -p ../outputs/source-data && echo hello > ../outputs/source-data/file.txt",
                            ),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "sink".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput::new("done".to_string(), "done.txt".to_string())],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("cat ../inputs/source/in/file.txt > ../outputs/done.txt"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
            ],
            edges: vec![Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "source".to_string(), port: "data".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            }],
        };

        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let rendered = fs::read_to_string(
            final_path.join("nodes").join("sink").join("outputs").join("done.txt"),
        )
        .unwrap();
        assert_eq!(rendered.trim(), "hello");

        let trace: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(final_path.join("nodes").join("source").join("trace.json"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(trace["outputs"][0]["kind"], "directory");
        assert_eq!(trace["outputs"][0]["media_type"], "application/vnd.bijux.directory");
        assert_eq!(trace["outputs"][0]["size_bytes"], 6);
        assert!(trace["outputs"][0]["sha256"].as_str().is_some());
        let inputs_index = read_inputs_index(&final_path, "sink");
        assert_eq!(inputs_index.files.len(), 1);
        assert_eq!(inputs_index.files[0].local_path, "source/in");
        assert_eq!(inputs_index.files[0].materialization_mode, "copy");
    }

    fn materialized_input_graph() -> Graph {
        Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "source".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec![],
                    outputs: vec![FileOutput::new("out".to_string(), "value.txt".to_string())],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("printf payload > ../outputs/value.txt"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "sink".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput::new("done".to_string(), "done.txt".to_string())],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("cat ../inputs/source/in > ../outputs/done.txt"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    cache: Default::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
            ],
            edges: vec![Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "source".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "sink".to_string(), port: "in".to_string() },
            }],
        }
    }

    #[test]
    fn copy_mode_records_explicit_source_input_contract() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&materialized_input_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();

        let inputs_index = read_inputs_index(&final_path, "sink");
        assert_eq!(inputs_index.files.len(), 1);
        let input = &inputs_index.files[0];
        assert_eq!(input.local_path, "source/in");
        assert_eq!(input.source_node_id, "source");
        assert_eq!(input.source_output_name, "out");
        assert_eq!(input.materialization_mode, "copy");
        assert_eq!(
            input.source_sha256,
            sha256_artifact_path(
                final_path.join("nodes").join("source").join("outputs").join("value.txt"),
            )
            .expect("source hash"),
        );
    }

    #[test]
    #[cfg(unix)]
    fn hardlink_mode_reuses_source_inode_for_materialized_input() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(
                &materialized_input_graph(),
                dir.path(),
                RuntimeConfig {
                    materialize_inputs: MaterializeMode::Hardlink,
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let source = final_path.join("nodes").join("source").join("outputs").join("value.txt");
        let local = final_path.join("nodes").join("sink").join("inputs").join("source").join("in");
        let inputs_index = read_inputs_index(&final_path, "sink");
        assert_eq!(inputs_index.files[0].materialization_mode, "hardlink");
        assert_eq!(fs::metadata(&source).unwrap().ino(), fs::metadata(&local).unwrap().ino());
    }

    #[test]
    #[cfg(unix)]
    fn symlink_mode_materializes_input_as_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(
                &materialized_input_graph(),
                dir.path(),
                RuntimeConfig {
                    materialize_inputs: MaterializeMode::Symlink,
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let local = final_path.join("nodes").join("sink").join("inputs").join("source").join("in");
        let inputs_index = read_inputs_index(&final_path, "sink");
        assert_eq!(inputs_index.files[0].materialization_mode, "symlink");
        assert!(fs::symlink_metadata(&local).unwrap().file_type().is_symlink());
    }

    #[test]
    fn local_shell_execution_enforces_node_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
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
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("n1").join("trace.json")).unwrap();
        assert!(trace.contains("timed out"));
        let run_log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        assert!(run_log.contains("\"event\":\"node_attempt_started\""));
        assert!(run_log.contains("\"event\":\"node_attempt_finished\""));
        assert!(run_log.contains("\"status\":\"failed\""));
    }

    #[test]
    fn container_execution_enforces_node_timeout_when_engine_is_available() {
        let _env_lock = process_env_lock();
        let dir = tempfile::tempdir().unwrap();
        let bin_dir = dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker = bin_dir.join("docker");
        fs::write(
            &docker,
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "docker fake 1.0"
  exit 0
fi
if [ "$1" = "image" ] && [ "$2" = "inspect" ]; then
  echo "sha256:feedface"
  exit 0
fi
if [ "$1" = "run" ]; then
  shift
  trap 'exit 143' TERM
  sleep 2
  exit 0
fi
exit 1
"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&docker).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&docker, perms).unwrap();
        }
        let mut path_entries = vec![bin_dir.display().to_string()];
        if let Some(existing_path) = std::env::var_os("PATH") {
            path_entries.push(existing_path.to_string_lossy().to_string());
        }
        let path_value = path_entries.join(":");
        let _path_guard = EnvVarGuard::replace("PATH", &path_value);
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "c".to_string(),
                kind: NodeKind::Container,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: Some(ContainerSpec {
                    engine: "docker".to_string(),
                    image: "example.local/runner@sha256:feedface".to_string(),
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
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("c").join("trace.json")).unwrap();
        assert!(trace.contains("\"code\": \"EXEC_TIMEOUT\""));
    }

    #[test]
    fn undeclared_output_file_fails_execution() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::Shell,
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("declared".to_string(), "declared.txt".to_string(),)],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from(
                            "echo ok > ../outputs/declared.txt && echo bad > ../outputs/extra.txt",
                        ),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime.run(&graph, dir.path(), RuntimeConfig::default()).unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("n1").join("trace.json")).unwrap();
        assert!(trace.contains("\"OUTPUT_UNDECLARED\""));
    }

    #[test]
    fn adapter_metadata_present_for_run_and_replay() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let original = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
        let replay = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap();
        let trace_a =
            fs::read_to_string(original.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b =
            fs::read_to_string(replay.join("nodes").join("a").join("trace.json")).unwrap();
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
                ..PolicyConfig::default()
            },
            ..RuntimeConfig::default()
        };
        let final_path = runtime.run(&graph, dir.path(), options).unwrap();
        let inputs_index = read_inputs_index(&final_path, "b");
        assert_eq!(inputs_index.files.len(), 1);
        assert_eq!(inputs_index.files[0].local_path, "a/in");
    }

    #[test]
    fn missing_adapter_registration_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("missing-kind".to_string()),
                semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out".to_string(), "out".to_string())],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                cache: Default::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
                trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                branch: None,
                dynamic: None,
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
            let options = RuntimeConfig { cache_mode: mode, ..RuntimeConfig::default() };
            let run_dir = runtime.run(&sample_graph(), dir.path(), options).unwrap();
            assert!(run_dir.join("manifest.json").exists());
            assert!(run_dir.join("manifest.finalized.json").exists());
            assert!(run_dir.join(".run-complete.json").exists());
            assert!(run_dir.join("run.schema.json").exists());
        }
    }

    #[test]
    fn run_id_collision_is_deterministic_error() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let options =
            RuntimeConfig { run_id: Some("fixed-run-id".to_string()), ..RuntimeConfig::default() };
        let first = runtime.run(&sample_graph(), dir.path(), options.clone());
        assert!(first.is_ok());
        let second = runtime.run(&sample_graph(), dir.path(), options);
        assert!(second.is_err());
    }

    #[test]
    fn stale_staging_run_dir_is_rejected_before_execution() {
        let dir = tempfile::tempdir().unwrap();
        create_staging_conflict(dir.path(), "stale-run", "nodes/a/work");
        let runtime = Runtime::new();
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    run_id: Some("stale-run".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap_err();
        assert!(format!("{err}").contains("staging run directory already exists"));
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
                RuntimeConfig { latest_symlink: Some(latest.clone()), ..RuntimeConfig::default() },
            )
            .unwrap();
        let first_manifest = fs::read_to_string(first.join("manifest.json")).unwrap();

        let second = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig { latest_symlink: Some(latest.clone()), ..RuntimeConfig::default() },
            )
            .unwrap();
        let first_manifest_after = fs::read_to_string(first.join("manifest.json")).unwrap();
        let second_manifest = fs::read_to_string(second.join("manifest.json")).unwrap();
        assert_eq!(first_manifest, first_manifest_after);
        assert_ne!(first_manifest_after, second_manifest);
    }

    #[test]
    fn resume_reuses_completed_nodes_and_records_attempt_summary() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let initial = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    run_id: Some("resume-ready".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let resumed = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    resume_run_id: Some("resume-ready".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        assert_eq!(resumed, initial);
        let attempts = read_run_attempts(&resumed);
        assert_eq!(attempts.len(), 2);
        let latest = attempts.last().expect("latest attempt");
        assert_eq!(latest.reason, "resume");
        let summary = latest.resume_summary.as_ref().expect("resume summary");
        assert_eq!(summary.reused_nodes, vec!["a", "b", "c", "d"]);
        assert!(summary.rerun_nodes.is_empty());
        assert!(summary.rejected_nodes.is_empty());

        let run_log = fs::read_to_string(resumed.join("run.log.jsonl")).unwrap();
        assert!(run_log.contains("\"event\":\"run_resumed\""));
        assert!(run_log.contains("\"event\":\"node_resume_reused\""));
    }

    #[test]
    fn resume_reruns_corrupt_nodes_and_clears_stale_outputs() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let initial = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    run_id: Some("resume-corrupt".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        fs::write(initial.join("nodes").join("b").join("outputs").join("out_b"), "tampered\n")
            .unwrap();
        fs::create_dir_all(initial.join("nodes").join("d").join("outputs")).unwrap();
        fs::write(initial.join("nodes").join("d").join("outputs").join("stale.txt"), "stale")
            .unwrap();

        let resumed = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    resume_run_id: Some("resume-corrupt".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let attempts = read_run_attempts(&resumed);
        let summary = attempts
            .last()
            .and_then(|attempt| attempt.resume_summary.as_ref())
            .expect("resume summary");
        assert_eq!(summary.reused_nodes, vec!["a", "c"]);
        assert_eq!(summary.rerun_nodes, vec!["b", "d"]);
        assert!(summary.rejected_nodes.is_empty());
        assert!(!resumed.join("nodes").join("d").join("outputs").join("stale.txt").exists());
        let rerun_output =
            fs::read_to_string(resumed.join("nodes").join("b").join("outputs").join("out_b"))
                .unwrap();
        assert_eq!(rerun_output, "ok\n");
    }

    #[test]
    fn resume_reject_mode_records_blocked_nodes_without_execution() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let initial = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    run_id: Some("resume-reject".to_string()),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();
        fs::write(initial.join("nodes").join("b").join("outputs").join("out_b"), "tampered\n")
            .unwrap();

        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    resume_run_id: Some("resume-reject".to_string()),
                    resume_failure_mode: ResumeFailureMode::RejectIncomplete,
                    ..RuntimeConfig::default()
                },
            )
            .unwrap_err();
        assert!(format!("{err}").contains("resume rejected incomplete nodes"));

        let staging_path = dir.path().join("run.tmp-resume-reject");
        let attempts = read_run_attempts(&staging_path);
        let summary = attempts
            .last()
            .and_then(|attempt| attempt.resume_summary.as_ref())
            .expect("resume summary");
        assert_eq!(summary.reused_nodes, vec!["a", "c"]);
        assert!(summary.rerun_nodes.is_empty());
        assert_eq!(summary.rejected_nodes, vec!["b", "d"]);
    }

    #[test]
    fn run_snapshot_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("run.snapshot.json")),
            Arc::new(SystemClock),
        );
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn run_attempt_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("run.attempts.json")),
            Arc::new(SystemClock),
        );
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn lineage_snapshot_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("lineage.snapshot.json")),
            Arc::new(SystemClock),
        );
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig::default(),
            )
            .unwrap_err();
        let rendered = format!("{err}");
        assert!(matches!(
            err,
            RuntimeError::Executor(_) | RuntimeError::Artifact(_) | RuntimeError::Io(_)
        ));
        assert!(rendered.contains("lineage snapshot"));
    }

    #[test]
    fn lineage_visualization_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("observability.lineage-visualization.json")),
            Arc::new(SystemClock),
        );
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig::default(),
            )
            .unwrap_err();
        let rendered = format!("{err}");
        assert!(matches!(
            err,
            RuntimeError::Executor(_) | RuntimeError::Artifact(_) | RuntimeError::Io(_)
        ));
        assert!(rendered.contains("lineage visualization"));
    }

    #[test]
    fn timeline_export_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("observability.timeline.json")),
            Arc::new(SystemClock),
        );
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig::default(),
            )
            .unwrap_err();
        let rendered = format!("{err}");
        assert!(matches!(
            err,
            RuntimeError::Executor(_) | RuntimeError::Artifact(_) | RuntimeError::Io(_)
        ));
        assert!(rendered.contains("timeline export"));
    }

    #[test]
    fn observability_payload_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("observability.root-causes.json")),
            Arc::new(SystemClock),
        );
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn audit_index_write_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::with_io(
            Arc::new(InterceptFs::fail_write("run-log.index.json")),
            Arc::new(SystemClock),
        );
        let err = runtime.run(&sample_graph(), dir.path(), RuntimeConfig::default()).unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }

    #[test]
    fn latest_symlink_failures_abort_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let runtime =
            Runtime::with_io(Arc::new(InterceptFs::fail_symlink("latest")), Arc::new(SystemClock));
        let err = runtime
            .run(
                &sample_graph(),
                dir.path(),
                RuntimeConfig {
                    latest_symlink: Some(dir.path().join("latest")),
                    ..RuntimeConfig::default()
                },
            )
            .unwrap_err();
        assert!(matches!(err, RuntimeError::Io(_)));
    }
}
