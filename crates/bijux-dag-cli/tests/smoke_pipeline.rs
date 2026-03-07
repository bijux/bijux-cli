use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;

use std::path::PathBuf;
use std::process::Command;

fn dag_command() -> Command {
    if let Some(path) = option_env!("CARGO_BIN_EXE_bijux") {
        if std::path::Path::new(path).exists() {
            return Command::new(path);
        }
    }

    let mut command = Command::new("cargo");
    command.env("CARGO_TARGET_DIR", "artifacts/target");
    command.args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--bin",
        "bijux",
        "--",
    ]);
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

#[test]
fn cli_smoke_minimal_pipeline_validate_plan_run_replay_diff() {
    let dag = write_temp_dag();
    let run_out = tempfile::tempdir().expect("run output dir");
    let replay_out = tempfile::tempdir().expect("replay output dir");

    let validate = dag_command()
        .args(["dag", "validate", dag.to_str().expect("dag path")])
        .output()
        .expect("validate output");
    assert!(validate.status.success(), "validate failed");

    let plan = dag_command()
        .args([
            "dag",
            "show-effective-plan",
            dag.to_str().expect("dag path"),
        ])
        .output()
        .expect("plan output");
    assert!(plan.status.success(), "show-effective-plan failed");

    let run = dag_command()
        .args([
            "dag",
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
            "dag",
            "replay",
            first_run_dir.to_str().expect("run dir path"),
            "--out",
            replay_out.path().to_str().expect("replay out path"),
        ])
        .output()
        .expect("replay output");
    assert!(replay.status.success(), "replay failed");

    let replay_run_dir = newest_run_dir(replay_out.path());

    let diff = dag_command()
        .args([
            "dag",
            "diff",
            first_run_dir.to_str().expect("first run path"),
            replay_run_dir.to_str().expect("replay run path"),
        ])
        .output()
        .expect("diff output");
    assert!(diff.status.success(), "diff failed");
}
