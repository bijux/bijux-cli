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
fn dag_root_help_describes_release_boundary() {
    let mut buffer = Vec::new();
    dag_command().write_long_help(&mut buffer).expect("render help");
    let rendered = String::from_utf8(buffer).expect("utf8 help");

    assert!(rendered
        .contains("Validate, run, replay, explain, and compare reproducible computation graphs"));
    assert!(rendered.contains("v0.4.0 surface truth table:"));
    assert!(rendered.contains("stable: validate, plan, run, replay, runs ..., artifact, artifact-inspect, diff, explain, verify, doctor, cache, version, commands"));
    assert!(rendered.contains("commands --lane experimental"));
    assert!(rendered.contains("commands --lane simulated"));
    assert!(rendered.contains("commands --lane internal"));
    assert!(rendered.contains("BIJUX_DAG_ENABLE_SIMULATED=1"));
    assert!(rendered.contains("BIJUX_DAG_ENABLE_INTERNAL=1"));
    assert!(rendered.contains("Use `bijux-dag commands` for the stable operator surface"));
    assert!(!rendered.contains("enterprise"));
    assert!(!rendered.contains("governance"));
    assert!(
        !rendered.contains("Validate, run, replay, and inspect reproducible computation graphs")
    );
}

#[test]
fn run_help_describes_human_and_json_progress_modes() {
    let mut run_command = dag_command().find_subcommand("run").expect("run subcommand").clone();
    let mut buffer = Vec::new();
    run_command.write_long_help(&mut buffer).expect("render run help");
    let rendered = String::from_utf8(buffer).expect("utf8 help");

    assert!(rendered.contains("show live progress for `bijux-dag run`"));
    assert!(rendered.contains("operator-readable updates on stderr"));
    assert!(rendered.contains("streams `dag.run.progress` JSON lines on stdout"));
}

#[test]
fn invalid_input_path_does_not_panic() {
    let result = std::panic::catch_unwind(|| {
        let cmd = dag_command();
        let matches = cmd
            .try_get_matches_from(["bijux-dag", "validate", "/definitely/missing/file.json"])
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
            .try_get_matches_from(["bijux-dag", "status", run.to_string_lossy().as_ref()])
            .expect("clap parse");
        let _ = dag_run(&matches);
    });
    assert!(result.is_ok());
}

#[test]
fn upstream_target_flags_parse_on_plan_and_run_surfaces() {
    let plan_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "plan",
            "explain",
            "./graph.json",
            "--to-node",
            "publish",
        ])
        .expect("plan parse");
    assert_eq!(dag_run(&plan_matches).expect_err("missing graph"), std::process::ExitCode::from(3));

    let run_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "run",
            "./graph.json",
            "--out",
            "./runs",
            "--to-node",
            "publish",
        ])
        .expect("run parse");
    assert_eq!(dag_run(&run_matches).expect_err("missing graph"), std::process::ExitCode::from(3));
}

#[test]
fn graph_inspection_selector_flags_parse_for_dag_inputs() {
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "show-effective-graph",
            "./graph.json",
            "--select",
            "id:publish",
            "--dependency-closure",
        ])
        .expect("graph inspection parse");
    assert_eq!(dag_run(&matches).expect_err("missing graph"), std::process::ExitCode::from(3));
}

#[test]
fn graph_inspection_rejects_run_dir_with_selector_overlay() {
    let result = dag_command().try_get_matches_from([
        "bijux-dag",
        "show-effective-graph",
        "--run-dir",
        "./runs/run-123",
        "--select",
        "id:publish",
    ]);
    assert!(result.is_err());
}
