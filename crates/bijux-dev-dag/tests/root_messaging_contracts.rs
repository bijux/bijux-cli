use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn readme_uses_canonical_mission_one_liner() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("README.md")).expect("read README");
    let mission =
        fs::read_to_string(root.join("docs/spec/MISSION_STATEMENT.md")).expect("read mission spec");

    assert!(
        mission.contains("`Git for computation graphs.`"),
        "mission spec must define canonical one-liner"
    );
    assert!(
        readme.contains("Git for computation graphs."),
        "README must include canonical one-liner"
    );
}

#[test]
fn readme_does_not_oversell_platform_scope() {
    let root = repo_root();
    let readme = fs::read_to_string(root.join("README.md")).expect("read README");
    let banned = [
        "full platform",
        "production-ready distributed orchestration",
        "drop-in replacement for Airflow",
        "drop-in replacement for Prefect",
        "drop-in replacement for Dagster",
    ];
    for token in banned {
        assert!(
            !readme.contains(token),
            "README contains oversell term banned by root messaging contract: {token}"
        );
    }
}

#[test]
fn root_messaging_contract_and_support_policy_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/ROOT_MESSAGING_CONTRACT.md",
        "docs/reference/EXECUTION_SUPPORT_POLICY.md",
        "docs/reference/POSITIONING_NOTE.md",
        "docs/reference/ROOT_CAPABILITY_MATRIX.md",
    ] {
        assert!(
            root.join(required).exists(),
            "missing root messaging/scope contract surface: {required}"
        );
    }
}
