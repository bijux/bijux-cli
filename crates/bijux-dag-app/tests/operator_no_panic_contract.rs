use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
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
use tar as _;
use tempfile as _;
use thiserror as _;

#[test]
fn operator_commands_do_not_panic_on_corrupt_run_dirs() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-bad");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(run.join("manifest.json"), "{bad-json").expect("write");

    let commands = vec![
        vec!["bijux-dag", "--json", "runs", "inspect", "run-bad", "--root", root.to_str().unwrap()],
        vec!["bijux-dag", "--json", "runs", "show", "run-bad", "--root", root.to_str().unwrap()],
        vec![
            "bijux-dag",
            "--json",
            "runs",
            "timeline",
            "run-bad",
            "--root",
            root.to_str().unwrap(),
        ],
        vec!["bijux-dag", "--json", "runs", "tree", "run-bad", "--root", root.to_str().unwrap()],
        vec![
            "bijux-dag",
            "--json",
            "runs",
            "explain-failure",
            "run-bad",
            "--root",
            root.to_str().unwrap(),
        ],
        vec!["bijux-dag", "--json", "trace-artifact", run.to_str().unwrap(), "a:b.txt"],
    ];

    for cmd in commands {
        let matches = dag_command().try_get_matches_from(cmd).expect("parse");
        let _ = dag_run(&matches);
    }
}
