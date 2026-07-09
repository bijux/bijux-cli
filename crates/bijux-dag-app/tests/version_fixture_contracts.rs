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
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn run_with_internal_lane(
    matches: &clap::ArgMatches,
) -> Result<std::process::ExitCode, std::process::ExitCode> {
    let previous = std::env::var_os("BIJUX_DAG_ENABLE_INTERNAL");
    std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", "1");
    let result = dag_run(matches);
    if let Some(value) = previous {
        std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", value);
    } else {
        std::env::remove_var("BIJUX_DAG_ENABLE_INTERNAL");
    }
    result
}

#[test]
fn supported_and_unsupported_graph_schema_fixtures_are_classified() {
    let root = repo_root();
    let supported = root.join("evidence/compat/graph_schema/v0_1_supported/minimal.dag.json");
    let unsupported_newer_version =
        root.join("evidence/compat/graph_schema/unsupported_newer_version/minimal.dag.json");

    let cmd = dag_command();
    let ok_matches = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--dag",
            supported.to_string_lossy().as_ref(),
        ])
        .expect("parse args supported");
    assert!(run_with_internal_lane(&ok_matches).is_ok());

    let bad_matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--dag",
            unsupported_newer_version.to_string_lossy().as_ref(),
        ])
        .expect("parse args unsupported");
    assert!(run_with_internal_lane(&bad_matches).is_err());
}

#[test]
fn supported_and_unsupported_run_dir_formats_are_classified() {
    let root = repo_root();
    let supported = root.join("evidence/compat/run_dir/v0_1_supported");
    let unsupported_newer_version = root.join("evidence/compat/run_dir/unsupported_newer_version");

    let cmd = dag_command();
    let ok_matches = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--run-dir",
            supported.to_string_lossy().as_ref(),
        ])
        .expect("parse args supported run");
    assert!(run_with_internal_lane(&ok_matches).is_ok());

    let bad_matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--run-dir",
            unsupported_newer_version.to_string_lossy().as_ref(),
        ])
        .expect("parse args unsupported run");
    assert!(run_with_internal_lane(&bad_matches).is_err());
}

#[test]
fn supported_and_unsupported_export_bundle_versions_are_classified() {
    let root = repo_root();
    let supported = root.join("evidence/compat/export_bundle/v0_1_supported/bundle.json");
    let unsupported_older_version =
        root.join("evidence/compat/export_bundle/unsupported_older_version/bundle.json");

    let cmd = dag_command();
    let ok_matches = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "import",
            supported.to_string_lossy().as_ref(),
        ])
        .expect("parse args supported bundle");
    assert!(dag_run(&ok_matches).is_ok());

    let bad_matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "import",
            unsupported_older_version.to_string_lossy().as_ref(),
        ])
        .expect("parse args unsupported bundle");
    assert!(dag_run(&bad_matches).is_err());
}
