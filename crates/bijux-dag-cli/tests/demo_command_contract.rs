use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;
use tempfile as _;

use std::path::PathBuf;
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn dag_demo_command_proves_retained_file_processing_workflow() {
    let root = repo_root();
    let demo_root = tempfile::tempdir().expect("demo root");
    let script = root.join("makes/bin/run_file_processing_demo.sh");
    let output = Command::new("/bin/bash")
        .arg(script)
        .current_dir(&root)
        .env("BIJUX_DAG_BIN", env!("CARGO_BIN_EXE_bijux-dag"))
        .env("BIJUX_DAG_DEMO_ROOT", demo_root.path())
        .output()
        .expect("run dag demo command");

    assert!(
        output.status.success(),
        "dag demo command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    assert!(
        stdout.contains("dag demo completed"),
        "dag demo output must report successful completion"
    );

    assert!(
        demo_root.path().join("runs/run-file-processing-cold").exists(),
        "cold run directory must be materialized"
    );
    assert!(
        demo_root.path().join("runs/run-file-processing-warm").exists(),
        "warm run directory must be materialized"
    );
    assert!(
        demo_root.path().join("runs/run-file-processing-replay").exists(),
        "replay run directory must be materialized"
    );
    assert!(
        demo_root
            .path()
            .join("runs/run-file-processing-cold/nodes/render_report/outputs/report/report.md")
            .exists(),
        "retained report artifact must be materialized"
    );

    let verify: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(demo_root.path().join("verify.json")).expect("read verify json"),
    )
    .expect("parse verify json");
    assert_eq!(verify["data"]["status"], "ok");
    assert_eq!(verify["data"]["mode"], "strict");
}
