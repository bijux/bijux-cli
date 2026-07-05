use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn dag_command() -> Command {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_bijux-dag") {
        if std::path::Path::new(&path).exists() {
            return Command::new(path);
        }
    }

    if let Some(path) = option_env!("CARGO_BIN_EXE_bijux-dag") {
        if std::path::Path::new(path).exists() {
            return Command::new(path);
        }
    }

    let root = repo_root();
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.env("CARGO_TARGET_DIR", root.join("artifacts/target"));
    command.env("LLVM_PROFILE_FILE", root.join("artifacts/coverage/profraw/default_%m_%p.profraw"));
    command.args(["run", "--quiet", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--"]);
    command
}

fn write_temp_dag() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-dag-cli-smoke-{}.json",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));

    let content = r#"{
  "spec": "bijux-dag/v0.1",
  "nodes": [
    {
      "id": "const1",
      "kind": "const",
      "inputs": [],
      "outputs": [{"name": "value", "path": "value.txt"}],
      "params": {"value": "hello"}
    }
  ],
  "edges": []
}
"#;

    std::fs::write(&path, content).expect("write dag");
    path
}

fn newest_run_dir(base: &std::path::Path) -> PathBuf {
    let mut entries: Vec<_> = std::fs::read_dir(base)
        .expect("read run output directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    entries.last().expect("at least one run directory").path()
}

fn run_id_from_run_dir(run_dir: &std::path::Path) -> String {
    run_dir.file_name().and_then(|name| name.to_str()).expect("run id").to_string()
}

fn first_artifact_id(run_dir: &std::path::Path) -> String {
    let index =
        std::fs::read_to_string(run_dir.join("outputs/index.json")).expect("read outputs index");
    let payload: serde_json::Value = serde_json::from_str(&index).expect("parse outputs index");
    let node_id = payload["files"][0]["node_id"].as_str().expect("node_id");
    let path = payload["files"][0]["path"].as_str().expect("path");
    let output_name =
        std::path::Path::new(path).file_name().and_then(|name| name.to_str()).expect("output name");
    format!("{node_id}:{output_name}")
}

#[test]
fn cli_smoke_minimal_pipeline_validate_plan_run_replay_diff() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");
    let replay_out = tempfile::tempdir().expect("replay output dir");

    let validate = dag_command()
        .args(["validate", dag.to_str().expect("dag path")])
        .output()
        .expect("validate output");
    assert!(validate.status.success(), "validate failed");

    let plan = dag_command()
        .args(["show-effective-plan", dag.to_str().expect("dag path")])
        .output()
        .expect("plan output");
    assert!(plan.status.success(), "show-effective-plan failed");

    let run = dag_command()
        .args([
            "run",
            dag.to_str().expect("dag path"),
            "--out",
            run_out.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run output");
    assert!(run.status.success(), "run failed");

    let first_run_dir = newest_run_dir(run_out.path());

    let replay = dag_command()
        .args([
            "replay",
            first_run_dir.to_str().expect("run dir path"),
            "--out",
            replay_out.path().to_str().expect("replay out path"),
        ])
        .output()
        .expect("replay output");
    assert!(replay.status.success(), "replay failed");

    let replay_run_dir = newest_run_dir(replay_out.path());
    let first_run_id = run_id_from_run_dir(&first_run_dir);

    let inspect = dag_command()
        .args([
            "runs",
            "inspect",
            "--root",
            run_out.path().to_str().expect("run root path"),
            first_run_id.as_str(),
            "--json",
        ])
        .output()
        .expect("inspect output");
    assert!(inspect.status.success(), "inspect failed");

    let diff = dag_command()
        .args([
            "diff",
            first_run_dir.to_str().expect("first run path"),
            replay_run_dir.to_str().expect("replay run path"),
        ])
        .output()
        .expect("diff output");
    assert!(diff.status.success(), "diff failed");
}

#[test]
fn cli_smoke_artifact_inspect_and_verify() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");

    let run = dag_command()
        .args([
            "run",
            dag.to_str().expect("dag path"),
            "--out",
            run_out.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run output");
    assert!(run.status.success(), "run failed");

    let run_dir = newest_run_dir(run_out.path());
    let artifact_id = first_artifact_id(&run_dir);

    let inspect = dag_command()
        .args([
            "artifact-inspect",
            run_dir.to_str().expect("run dir path"),
            artifact_id.as_str(),
            "--json",
        ])
        .output()
        .expect("artifact inspect output");
    assert!(inspect.status.success(), "artifact-inspect failed");

    let verify = dag_command()
        .args(["verify", run_dir.to_str().expect("run dir path"), "--deep", "--json"])
        .output()
        .expect("verify output");
    assert!(verify.status.success(), "verify failed");
}

#[test]
fn cli_smoke_export_import_and_fsck_verify_only() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");
    let bundle_file = run_out.path().join("run.bundle.json");

    let run = dag_command()
        .args([
            "run",
            dag.to_str().expect("dag path"),
            "--out",
            run_out.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run output");
    assert!(run.status.success(), "run failed");

    let run_dir = newest_run_dir(run_out.path());

    let export = dag_command()
        .args([
            "export",
            run_dir.to_str().expect("run dir path"),
            "--out",
            bundle_file.to_str().expect("bundle path"),
            "--json",
        ])
        .output()
        .expect("export output");
    assert!(export.status.success(), "export failed");

    let import = dag_command()
        .args(["import", bundle_file.to_str().expect("bundle path"), "--verify-only", "--json"])
        .output()
        .expect("import verify-only output");
    assert!(import.status.success(), "import verify-only failed");

    let fsck = dag_command()
        .args(["fsck", bundle_file.to_str().expect("bundle path"), "--json"])
        .output()
        .expect("fsck output");
    assert!(fsck.status.success(), "bundle fsck failed");
}

#[test]
fn cli_smoke_runs_history_list_show_timeline_and_tree() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");

    let run = dag_command()
        .args([
            "run",
            dag.to_str().expect("dag path"),
            "--out",
            run_out.path().to_str().expect("run out path"),
        ])
        .output()
        .expect("run output");
    assert!(run.status.success(), "run failed");

    let run_dir = newest_run_dir(run_out.path());
    let run_id = run_id_from_run_dir(&run_dir);

    for args in [
        vec!["runs", "list", "--root", run_out.path().to_str().expect("run out path"), "--json"],
        vec![
            "runs",
            "show",
            "--root",
            run_out.path().to_str().expect("run out path"),
            run_id.as_str(),
            "--json",
        ],
        vec!["runs", "history", "--root", run_out.path().to_str().expect("run out path"), "--json"],
        vec![
            "runs",
            "timeline",
            "--root",
            run_out.path().to_str().expect("run out path"),
            run_id.as_str(),
            "--json",
        ],
        vec![
            "runs",
            "tree",
            "--root",
            run_out.path().to_str().expect("run out path"),
            run_id.as_str(),
            "--json",
        ],
    ] {
        let output = dag_command().args(args).output().expect("runs flow output");
        assert!(output.status.success(), "runs flow failed");
    }
}
