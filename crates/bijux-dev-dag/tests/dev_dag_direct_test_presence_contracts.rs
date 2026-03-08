use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn listed_helper_modules_contain_direct_unit_tests() {
    let root = repo_root();
    for rel in [
        "crates/bijux-dev-dag/src/commands/perf_evidence.rs",
        "crates/bijux-dev-dag/src/commands/suite_catalog.rs",
        "crates/bijux-dev-dag/src/repo/layout.rs",
        "crates/bijux-dev-dag/src/repo/root.rs",
        "crates/bijux-dev-dag/src/report/write.rs",
        "crates/bijux-dev-dag/src/tooling/cargo.rs",
        "crates/bijux-dev-dag/src/tooling/git.rs",
        "crates/bijux-dev-dag/src/tooling/mod.rs",
    ] {
        let src = fs::read_to_string(root.join(rel)).expect("read module source");
        assert!(
            src.contains("#[cfg(test)]"),
            "missing unit-test module in {rel}"
        );
        assert!(src.contains("#[test]"), "missing direct test in {rel}");
    }
}

#[test]
fn dev_dag_helper_coverage_target_policy_exists() {
    let root = repo_root();
    let path = root.join("configs/policy/dev_dag_helper_coverage_targets.json");
    assert!(path.exists(), "missing helper coverage target policy");
    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read policy")).expect("json policy");
    assert!(
        payload
            .get("line_coverage_targets")
            .and_then(|v| v.as_object())
            .is_some_and(|v| !v.is_empty()),
        "line_coverage_targets must be a non-empty object"
    );
}
