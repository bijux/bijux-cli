use bijux_cli::api::runtime::run_app;
use bijux_dag_core::{
    canonical_json, lower_graph_to_execution_plan, parse_graph_strict, validate_graph, PlanOptions,
    Severity,
};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct HardReleaseGateContract {
    schema_version: String,
    fixture_workflow: String,
    required_steps: Vec<String>,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_contract() -> HardReleaseGateContract {
    let path = repo_root().join("contracts/foundation/hard_release_gate.v1.json");
    read_json(&path)
}

fn run_dag_command(args: &[&str], cwd: &Path) -> (i32, String, String) {
    let output = Command::new(resolve_bijux_dag_binary(cwd))
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("run dag command");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn run_dag_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag_command(args, cwd);
    assert!(code == 0, "command failed: code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse dag json envelope")
}

fn resolve_workspace_root(cwd: &Path) -> PathBuf {
    let mut current = cwd.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() && current.join("crates").exists() {
            return current;
        }
        if !current.pop() {
            panic!("unable to resolve workspace root from {}", cwd.display());
        }
    }
}

fn resolve_bijux_dag_binary(cwd: &Path) -> PathBuf {
    static BIN_PATH: OnceLock<PathBuf> = OnceLock::new();
    BIN_PATH
        .get_or_init(|| {
            if let Some(path) = std::env::var_os("BIJUX_DAG_BIN").map(PathBuf::from) {
                if path.exists() {
                    return path;
                }
            }
            let workspace_root = resolve_workspace_root(cwd);
            let target_root = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_root.join("artifacts").join("target"));
            let status = Command::new("cargo")
                .current_dir(&workspace_root)
                .env("RUSTFLAGS", "-Awarnings")
                .env("CARGO_TARGET_DIR", &target_root)
                .args(["build", "-q", "-p", "bijux-dag-cli"])
                .status()
                .expect("build bijux-dag binary");
            assert!(status.success(), "failed to build bijux-dag binary");
            target_root.join("debug").join(format!("bijux-dag{}", std::env::consts::EXE_SUFFIX))
        })
        .clone()
}

#[test]
fn hard_release_gate_contract_schema_is_current() {
    let contract = read_contract();
    assert_eq!(contract.schema_version, "foundation-hard-release-gate/v1");
    assert_eq!(contract.required_steps.len(), 10);
    assert!(contract.required_steps.iter().all(|step| !step.trim().is_empty()));
}

#[test]
fn hard_release_gate_exercises_root_routing_graph_lifecycle_and_evidence() {
    let root = repo_root();
    let contract = read_contract();
    let graph_path = root.join(&contract.fixture_workflow);
    let graph_raw =
        fs::read_to_string(&graph_path).unwrap_or_else(|err| panic!("read fixture failed: {err}"));

    let route_report = run_app(&[
        "bijux".to_string(),
        "inspect".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ])
    .expect("run root inspect");
    assert_eq!(route_report.exit_code, 0);
    let route_payload: Value = serde_json::from_str(&route_report.stdout).expect("parse inspect");
    assert!(route_payload["reserved_namespaces"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item["name"] == "dag")));

    let graph = parse_graph_strict(&graph_raw).expect("parse strict");
    let diagnostics = validate_graph(&graph);
    assert!(
        diagnostics.iter().all(|diag| diag.severity != Severity::Error),
        "validation produced errors: {diagnostics:?}"
    );
    let canonical = canonical_json(&graph).expect("canonical json");
    assert!(canonical.contains("\"nodes\""));
    let plan =
        lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan lowering");
    assert!(!plan.ordering.is_empty(), "plan ordering must not be empty");

    let tmp = tempdir().expect("tempdir");
    let out_dir = tmp.path().join("runs");
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&out_dir).expect("create run dir");
    fs::create_dir_all(&cache_dir).expect("create cache dir");
    let out = out_dir.to_string_lossy().into_owned();
    let cache = cache_dir.to_string_lossy().into_owned();
    let graph = graph_path.to_string_lossy().into_owned();

    let run = run_dag_json(
        &[
            "run",
            "--json",
            &graph,
            "--out",
            &out,
            "--run-id",
            "foundation-gate-source",
            "--cache",
            "readwrite",
            "--cache-dir",
            &cache,
        ],
        &root,
    );
    assert_eq!(run["ok"], true);
    let source_run = out_dir.join("run-foundation-gate-source");
    assert!(source_run.is_dir(), "source run dir missing");
    assert!(source_run.join("outputs").join("index.json").is_file(), "outputs index missing");

    let cache_keys = fs::read_dir(&cache_dir)
        .expect("read cache dir")
        .flatten()
        .filter_map(|entry| {
            entry.file_type().ok().and_then(|kind| {
                kind.is_dir().then(|| entry.file_name().to_string_lossy().into_owned())
            })
        })
        .collect::<Vec<_>>();
    assert!(!cache_keys.is_empty(), "cache entries were not materialized");

    let cache_explain = run_dag_json(
        &["cache", "explain", "--json", "--cache-dir", &cache, "--key", &cache_keys[0]],
        &root,
    );
    assert_eq!(cache_explain["ok"], true);
    assert!(cache_explain["data"]["key_components"].is_object());

    let source_run_str = source_run.to_string_lossy().into_owned();
    let replay = run_dag_json(
        &[
            "replay",
            "--json",
            &source_run_str,
            "--out",
            &out,
            "--run-id",
            "foundation-gate-replay",
            "--prove",
        ],
        &root,
    );
    assert_eq!(replay["ok"], true);
    assert!(replay["data"]["replay_proof"].is_object(), "missing replay proof");

    let replay_run = out_dir.join("run-foundation-gate-replay");
    assert!(replay_run.is_dir(), "replay run dir missing");
    let replay_run_str = replay_run.to_string_lossy().into_owned();
    let verify = run_dag_json(&["verify", "--json", &replay_run_str], &root);
    assert_eq!(verify["ok"], true, "verify command must succeed");
}
