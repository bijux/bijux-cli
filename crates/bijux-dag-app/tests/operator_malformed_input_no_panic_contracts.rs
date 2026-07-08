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
use std::panic::AssertUnwindSafe;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run, inspect_summary};
use std::fs;

#[test]
fn operator_inspect_handles_malformed_run_manifest_without_panicking() {
    let temp = tempfile::tempdir().expect("tempdir");
    let run = temp.path().join("run-bad-manifest");
    fs::create_dir_all(&run).expect("create run dir");
    fs::write(run.join("manifest.json"), b"{\"run_id\":\"broken\",")
        .expect("write broken manifest");
    fs::write(
        run.join("graph.snapshot.json"),
        b"{\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]}",
    )
    .expect("write snapshot");

    let result = std::panic::catch_unwind(AssertUnwindSafe(|| inspect_summary(&run)));
    assert!(result.is_ok(), "inspect summary panicked on malformed manifest");
}

#[test]
fn prove_and_verify_commands_handle_malformed_run_dir_without_panicking() {
    let temp = tempfile::tempdir().expect("tempdir");
    let run = temp.path().join("run-malformed");
    fs::create_dir_all(&run).expect("create run dir");
    fs::write(run.join("manifest.json"), b"{not-json").expect("write malformed manifest");
    fs::write(run.join("graph.snapshot.json"), b"{not-json").expect("write malformed snapshot");

    let prove = dag_command()
        .try_get_matches_from(["bijux-dag", "--json", "prove", run.to_str().expect("run path")])
        .expect("prove matches");
    let prove_result = std::panic::catch_unwind(AssertUnwindSafe(|| dag_run(&prove)));
    assert!(prove_result.is_ok(), "prove command panicked on malformed run dir");

    let verify = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "verify",
            run.to_str().expect("run path"),
            "--deep",
            "--strict",
        ])
        .expect("verify matches");
    let verify_result = std::panic::catch_unwind(AssertUnwindSafe(|| dag_run(&verify)));
    assert!(verify_result.is_ok(), "verify command panicked on malformed run dir");
}
