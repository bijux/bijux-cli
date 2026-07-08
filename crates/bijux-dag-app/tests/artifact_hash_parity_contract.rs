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
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};

mod support;

#[test]
#[ignore = "experimental"]
fn hash_artifact_cli_output_matches_internal_sha256() {
    let dir = tempfile::tempdir().expect("tmp");
    let file = dir.path().join("artifact.bin");
    fs::write(&file, b"artifact-hash-parity").expect("write artifact");

    let expected =
        bijux_dag_artifacts::hash::sha256_hex(&fs::read(&file).expect("read artifact bytes"));

    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "hash",
            "artifact",
            "--json",
            file.to_string_lossy().as_ref(),
        ])
        .expect("parse args");

    let code = dag_run(&matches).expect("hash artifact command");
    assert_eq!(code, std::process::ExitCode::SUCCESS);

    let file_lossy = file.to_string_lossy().to_string();
    let args = ["hash", "artifact", "--json", file_lossy.as_str()];
    let output = run_dag_json(&args, &repo_root());

    assert_eq!(output["command"], "dag.hash.artifact");
    assert_eq!(output["data"]["artifact_sha256"], expected);
}

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn run_dag_json(args: &[&str], cwd: &Path) -> serde_json::Value {
    let out = support::run_dag_command(args, cwd);
    assert_eq!(out.0, 0, "stderr={}", out.2);
    serde_json::from_str(&out.1).expect("json stdout")
}
