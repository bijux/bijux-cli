use bijux_dag_app::{dag_command, dag_run};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn supported_and_unsupported_graph_schema_fixtures_are_classified() {
    let root = repo_root();
    let supported = root.join("tests/compatibility/graph_schema/v0.1/minimal.dag.json");
    let unsupported_future = root.join("tests/compatibility/graph_schema/unsupported_future/minimal.dag.json");

    let cmd = dag_command();
    let ok_matches = cmd
        .clone()
        .try_get_matches_from([
            "dag",
            "--json",
            "version-inspect",
            "--dag",
            supported.to_string_lossy().as_ref(),
        ])
        .expect("parse args supported");
    assert!(dag_run(&ok_matches).is_ok());

    let bad_matches = cmd
        .try_get_matches_from([
            "dag",
            "--json",
            "version-inspect",
            "--dag",
            unsupported_future.to_string_lossy().as_ref(),
        ])
        .expect("parse args unsupported");
    assert!(dag_run(&bad_matches).is_err());
}

#[test]
fn supported_and_unsupported_run_dir_formats_are_classified() {
    let root = repo_root();
    let supported = root.join("tests/compatibility/run_dir/v0.1");
    let unsupported = root.join("tests/compatibility/run_dir/unsupported_future");

    let cmd = dag_command();
    let ok_matches = cmd
        .clone()
        .try_get_matches_from([
            "dag",
            "--json",
            "version-inspect",
            "--run-dir",
            supported.to_string_lossy().as_ref(),
        ])
        .expect("parse args supported run");
    assert!(dag_run(&ok_matches).is_ok());

    let bad_matches = cmd
        .try_get_matches_from([
            "dag",
            "--json",
            "version-inspect",
            "--run-dir",
            unsupported.to_string_lossy().as_ref(),
        ])
        .expect("parse args unsupported run");
    assert!(dag_run(&bad_matches).is_err());
}
