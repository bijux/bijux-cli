use bijux_dag_app::{dag_command, dag_run};

#[test]
fn config_show_effective_accepts_cli_overrides() {
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "dag",
            "config",
            "show-effective",
            "--jobs",
            "4",
            "--cache-mode",
            "read",
            "--materialize-inputs",
            "hardlink",
        ])
        .expect("parse config show-effective");
    let result = dag_run(&matches);
    assert!(result.is_ok());
}

#[test]
fn policy_show_effective_emits_trace_surface() {
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "dag",
            "--json",
            "policy",
            "show-effective",
            "--deny-network",
            "--clean-env",
            "--allow-env",
            "PATH",
        ])
        .expect("parse policy show-effective");
    let result = dag_run(&matches);
    assert!(result.is_ok());
}

#[test]
fn config_show_effective_rejects_malformed_file_before_execution() {
    let tmp = tempfile::tempdir().expect("tmp");
    let bad = tmp.path().join("bad-config.json");
    std::fs::write(&bad, "{not-json").expect("write bad json");

    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "dag",
            "config",
            "show-effective",
            "--config",
            bad.to_string_lossy().as_ref(),
        ])
        .expect("parse config show-effective with bad file");
    let result = dag_run(&matches);
    assert!(result.is_err());
}
