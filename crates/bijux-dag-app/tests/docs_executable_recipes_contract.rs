use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("canonical repo root")
}

fn load_recipe_commands(docs_path: &Path, recipe_id: &str) -> Vec<String> {
    let source = fs::read_to_string(docs_path).expect("read recipe docs");
    let start = format!("<!-- recipe:{recipe_id}:start -->");
    let end = format!("<!-- recipe:{recipe_id}:end -->");
    let block = source
        .split(&start)
        .nth(1)
        .and_then(|tail| tail.split(&end).next())
        .expect("recipe block markers");
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("bijux-dag "))
        .map(str::to_string)
        .collect()
}

fn substitute_vars(line: &str, vars: &BTreeMap<&str, String>) -> String {
    let mut rendered = line.to_string();
    for (key, value) in vars {
        rendered = rendered.replace(&format!("${{{key}}}"), value);
    }
    rendered
}

fn run_recipe_command(root: &Path, command: &str) -> Value {
    let args = command
        .strip_prefix("bijux-dag ")
        .expect("command prefix")
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) = support::run_dag_command(&arg_refs, root);
    assert_eq!(
        code, 0,
        "recipe command failed\ncommand: {command}\nstdout: {stdout}\nstderr: {stderr}"
    );
    if args.iter().any(|arg| arg == "--json") {
        serde_json::from_str(&stdout).expect("json output")
    } else {
        Value::Null
    }
}

#[test]
fn docs_major_dag_recipe_is_ci_executable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let run_root = temp.path().join("runs");
    let replay_root = temp.path().join("replays");
    let run_id = "recipe-main-run";
    let run_dir = run_root.join(run_id);
    let export_bundle = temp.path().join("export.tar.gz");
    let diagnostics_bundle = temp.path().join("diagnostics.json");
    fs::create_dir_all(&run_root).expect("run root");
    fs::create_dir_all(&replay_root).expect("replay root");

    let mut vars = BTreeMap::new();
    vars.insert(
        "GRAPH",
        root.join("evidence/dag/authoring/examples/hello.dag.json")
            .canonicalize()
            .expect("canonical graph fixture")
            .to_string_lossy()
            .into_owned(),
    );
    vars.insert("RUN_ROOT", run_root.to_string_lossy().into_owned());
    vars.insert("RUN_ID", run_id.to_string());
    vars.insert("RUN_DIR", run_dir.to_string_lossy().into_owned());
    vars.insert("REPLAY_ROOT", replay_root.to_string_lossy().into_owned());
    vars.insert("EXPORT_BUNDLE", export_bundle.to_string_lossy().into_owned());
    vars.insert("DIAG_BUNDLE", diagnostics_bundle.to_string_lossy().into_owned());

    let docs_path = root.join("docs/bijux-dag/interfaces/executable-recipes.md");
    let commands = load_recipe_commands(&docs_path, "ci-major-dag-commands");
    assert!(
        commands.len() >= 10,
        "expected major command recipe set, got {} commands",
        commands.len()
    );

    for command in commands {
        let rendered = substitute_vars(&command, &vars);
        let payload = run_recipe_command(&root, &rendered);
        if !payload.is_null() {
            assert!(payload.is_object(), "json mode must return a top-level object");
        }
        if rendered.starts_with("bijux-dag run ") {
            let run_dir = payload
                .get("data")
                .and_then(|data| data.get("run_dir"))
                .and_then(Value::as_str)
                .expect("run output run_dir");
            vars.insert("RUN_DIR", run_dir.to_string());
            let run_name = Path::new(run_dir)
                .file_name()
                .and_then(|name| name.to_str())
                .expect("run directory name");
            vars.insert("RUN_ID", run_name.to_string());
        }
    }

    let resolved_run_dir =
        PathBuf::from(vars.get("RUN_DIR").expect("resolved run directory variable"));
    assert!(resolved_run_dir.exists(), "run directory must be materialized");
    assert!(run_root.join(".bijux-run-history-index.json").exists(), "run index must be present");
    assert!(diagnostics_bundle.exists(), "diagnostics bundle must be exported");
    assert!(export_bundle.exists(), "export bundle must be exported");
}

#[test]
fn docs_evidence_backed_bulletin_recipe_is_ci_executable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let run_root = temp.path().join("runs");
    let cache_root = temp.path().join("cache");
    let deliverables_root = temp.path().join("deliverables");
    fs::create_dir_all(&run_root).expect("run root");
    fs::create_dir_all(&cache_root).expect("cache root");
    fs::create_dir_all(&deliverables_root).expect("deliverables root");

    let mut vars = BTreeMap::new();
    vars.insert(
        "GRAPH",
        root.join("evidence/dag/authoring/examples/audience-branch-bulletin.dag.json")
            .canonicalize()
            .expect("canonical graph fixture")
            .to_string_lossy()
            .into_owned(),
    );
    vars.insert("RUN_ROOT", run_root.to_string_lossy().into_owned());
    vars.insert("CACHE_ROOT", cache_root.to_string_lossy().into_owned());
    vars.insert("DELIVERABLES_ROOT", deliverables_root.to_string_lossy().into_owned());
    vars.insert(
        "SOURCE_NOTE",
        root.join("evidence/dag/authoring/examples/audience-branch-source/team-update.md")
            .canonicalize()
            .expect("canonical source note")
            .to_string_lossy()
            .into_owned(),
    );
    vars.insert(
        "REVISED_NOTE",
        root.join("evidence/dag/authoring/examples/audience-branch-source/team-update-revised.md")
            .canonicalize()
            .expect("canonical revised note")
            .to_string_lossy()
            .into_owned(),
    );

    let docs_path = root.join("docs/bijux-dag/interfaces/executable-recipes.md");
    let commands = load_recipe_commands(&docs_path, "ci-evidence-backed-bulletin");
    assert!(
        commands.len() >= 8,
        "expected evidence-backed bulletin recipe set, got {} commands",
        commands.len()
    );

    for command in commands {
        let rendered = substitute_vars(&command, &vars);
        let payload = run_recipe_command(&root, &rendered);
        if !payload.is_null() {
            assert!(payload.is_object(), "json mode must return a top-level object");
        }
    }

    assert!(
        run_root.join("run-branch-bulletin-replay").exists(),
        "replay run must be materialized"
    );
    assert!(
        deliverables_root
            .join("release/branch-bulletin-updated/publish_bulletin/bulletin/payload/bulletin.md")
            .exists(),
        "promoted bulletin must be materialized"
    );
}
