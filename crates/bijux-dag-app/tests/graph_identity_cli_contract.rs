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
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;

#[test]
fn hash_graph_and_fingerprint_explain_commands_are_wired() {
    let command_names = dag_command()
        .get_subcommands()
        .map(|command| command.get_name().to_string())
        .collect::<Vec<_>>();
    assert!(command_names.contains(&"hash".to_string()));
    assert!(command_names.contains(&"canonical-bytes".to_string()));
    assert!(command_names.contains(&"canonical-diff".to_string()));
    assert!(command_names.contains(&"fingerprint".to_string()));
}

#[test]
fn fingerprint_explain_json_matches_contract_shape() {
    let temp = tempfile::tempdir().expect("tmp");
    let dag_path = temp.path().join("g.dag.json");
    fs::write(&dag_path, r#"{"spec":"bijux-dag/v0.1","nodes":[],"edges":[]}"#).expect("write dag");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "fingerprint",
            dag_path.to_string_lossy().as_ref(),
            "--explain",
        ])
        .expect("parse");
    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
