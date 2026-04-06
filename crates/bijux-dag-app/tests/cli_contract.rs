use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};

#[test]
fn dag_help_command_tree_snapshot_is_stable() {
    let mut names =
        dag_command().get_subcommands().map(|c| c.get_name().to_string()).collect::<Vec<_>>();
    names.sort();
    let rendered = format!("{}\n", names.join("\n"));
    let expected = include_str!("snapshots/dag_command_tree.txt");
    assert_eq!(rendered, expected);
}

#[test]
fn invalid_input_path_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        let cmd = dag_command();
        let matches = cmd
            .try_get_matches_from(["dag", "validate", "/definitely/missing/file.json"])
            .expect("clap parse");
        let _ = dag_run(&matches);
    });
    assert!(result.is_ok());
}

#[test]
fn corrupted_run_dir_does_not_panic() {
    let dir = tempfile::tempdir().expect("tmpdir");
    let run = dir.path().join("run-bad");
    std::fs::create_dir_all(run.join("nodes")).expect("mkdir");
    std::fs::write(run.join("manifest.json"), "{not-json").expect("write manifest");

    let result = std::panic::catch_unwind(|| {
        let cmd = dag_command();
        let matches = cmd
            .try_get_matches_from(["dag", "status", run.to_string_lossy().as_ref()])
            .expect("clap parse");
        let _ = dag_run(&matches);
    });
    assert!(result.is_ok());
}
