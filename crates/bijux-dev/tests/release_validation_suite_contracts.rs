use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseValidationSuite {
    format: String,
    local_entrypoint: String,
    ci_entrypoint: String,
    maintainer_entrypoint: String,
    package_boundary_contract: String,
    documentation: ReleaseValidationDocumentation,
    release_tree: ReleaseTreeContract,
    verify_flow: Vec<String>,
    public_dag_crates: Vec<String>,
    commands: Vec<String>,
    artifacts: Vec<ReleaseValidationArtifact>,
    failure_ownership: Vec<ReleaseFailureOwnership>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseTreeContract {
    script: String,
    candidate_ref: String,
    version_source: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseValidationDocumentation {
    operator_handbook: String,
    workflow_handbook: String,
    release_operations: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseValidationArtifact {
    path: String,
    purpose: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ReleaseFailureOwnership {
    failure_class: String,
    owner: String,
    action: String,
}

#[derive(Debug, Deserialize)]
struct WorkspacePackageBoundary {
    packages: Vec<PackageBoundaryEntry>,
    crates_io_publish_order: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageBoundaryEntry {
    #[serde(rename = "crate")]
    crate_name: String,
    product_family: String,
    release_status: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn read_suite() -> ReleaseValidationSuite {
    let path = repo_root().join("configs/dag/release/release_validation_suite.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read suite failed: {err}"));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse suite failed: {err}"))
}

fn read_workspace_package_boundary(path: &str) -> WorkspacePackageBoundary {
    let absolute = repo_root().join(path);
    let raw = fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("read {} failed: {err}", absolute.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("parse {} failed: {err}", absolute.display()))
}

fn public_dag_crates_from_boundary(boundary: &WorkspacePackageBoundary) -> Vec<String> {
    let dag_public_crates = boundary
        .packages
        .iter()
        .filter(|entry| entry.product_family == "bijux-dag" && entry.release_status == "public")
        .map(|entry| entry.crate_name.as_str())
        .collect::<BTreeSet<_>>();
    boundary
        .crates_io_publish_order
        .iter()
        .filter(|crate_name| dag_public_crates.contains(crate_name.as_str()))
        .cloned()
        .collect()
}

fn read_public_dag_manifest(crate_name: &str) -> String {
    let path = repo_root().join("crates").join(crate_name).join("Cargo.toml");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()))
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("read {} failed: {err}", absolute.display()))
}

fn run_release_explain_verify() -> Value {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "-p",
            "bijux-dev",
            "--bin",
            "bijux-dev-dag",
            "--",
            "--json",
            "release",
            "explain",
            "--suite",
            "verify",
        ])
        .current_dir(repo_root())
        .output()
        .expect("run release explain");
    assert!(
        output.status.success(),
        "release explain failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse release explain json")
}

#[test]
fn release_validation_suite_is_current() {
    let suite = read_suite();
    let boundary = read_workspace_package_boundary(&suite.package_boundary_contract);
    assert_eq!(suite.format, "release-validation-suite/v1");
    assert_eq!(suite.local_entrypoint, "make release-validate-rs");
    assert_eq!(suite.ci_entrypoint, "make gh-release-validate");
    assert_eq!(
        suite.maintainer_entrypoint,
        "cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify"
    );
    assert_eq!(
        suite.package_boundary_contract,
        "contracts/foundation/workspace_package_boundary.v1.json"
    );
    assert_eq!(
        suite.documentation.operator_handbook,
        "docs/bijux-dev/operations/release-validation-suite.md"
    );
    assert_eq!(
        suite.documentation.workflow_handbook,
        "docs/bijux-dev/gh-workflows/release-validation.md"
    );
    assert_eq!(
        suite.documentation.release_operations,
        "docs/bijux-dev/operations/release-operations.md"
    );
    assert_eq!(suite.release_tree.script, ".github/scripts/prepare_release_tree.py");
    assert_eq!(suite.release_tree.candidate_ref, "HEAD");
    assert_eq!(suite.release_tree.version_source, "workspace.package.version");
    assert_eq!(
        suite.verify_flow,
        vec![
            "release.validation-suite".to_string(),
            "release.readiness".to_string(),
            "release.compatibility-matrix".to_string(),
        ]
    );
    assert_eq!(suite.public_dag_crates, public_dag_crates_from_boundary(&boundary));
    assert_eq!(suite.artifacts.len(), 5, "expected governed release artifact inventory");
    assert_eq!(suite.failure_ownership.len(), 3, "expected actionable failure ownership inventory");
}

#[test]
fn release_validation_suite_commands_cover_required_release_checks() {
    let suite = read_suite();
    let commands = suite.commands;
    assert_eq!(commands.len(), 15, "expected exact release validation command count");

    let required = [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
        "cargo test --workspace --all-targets --all-features --locked",
        "cargo doc --workspace --all-features --no-deps",
        "cargo package -p bijux-dag-core --list",
        "cargo package -p bijux-dag-artifacts --list",
        "cargo package -p bijux-dag-runtime --list",
        "cargo package -p bijux-dag-app --list",
        "cargo package -p bijux-dag-cli --list",
        "cargo publish -p bijux-dag-core --dry-run --locked",
        "cargo publish -p bijux-dag-artifacts --dry-run --locked",
        "cargo publish -p bijux-dag-runtime --dry-run --locked",
        "cargo publish -p bijux-dag-app --dry-run --locked",
        "cargo publish -p bijux-dag-cli --dry-run --locked",
        "cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture",
    ];

    for command in required {
        assert!(
            commands.iter().any(|entry| entry == command),
            "missing release validation command: {command}"
        );
    }

    let fmt_index = commands
        .iter()
        .position(|entry| entry == "cargo fmt --all -- --check")
        .expect("fmt command");
    let clippy_index = commands
        .iter()
        .position(|entry| {
            entry == "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings"
        })
        .expect("clippy command");
    let test_index = commands
        .iter()
        .position(|entry| entry == "cargo test --workspace --all-targets --all-features --locked")
        .expect("test command");
    let doc_index = commands
        .iter()
        .position(|entry| entry == "cargo doc --workspace --all-features --no-deps")
        .expect("doc command");
    let smoke_index = commands
        .iter()
        .position(|entry| {
            entry == "cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture"
        })
        .expect("smoke command");

    assert!(fmt_index < clippy_index);
    assert!(clippy_index < test_index);
    assert!(test_index < doc_index);
    assert!(doc_index < smoke_index);
}

#[test]
fn public_dag_manifests_do_not_depend_on_private_testkit_release() {
    let suite = read_suite();

    for crate_name in suite.public_dag_crates {
        let manifest = read_public_dag_manifest(&crate_name);
        assert!(
            !manifest.contains("bijux-dag-testkit"),
            "{crate_name} must not depend on the private bijux-dag-testkit crate because public package verification must not require unpublished support crates"
        );
    }
}

#[test]
fn release_validation_suite_entrypoints_are_wired_into_make_and_ci() {
    let gh_makefile = read_repo_file("makes/gh.mk");
    let rust_makefile = read_repo_file("makes/rust.mk");
    let workflow = read_repo_file(".github/workflows/release-validation.yml");
    let operations_doc = read_repo_file("docs/bijux-dev/operations/release-validation-suite.md");
    let workflow_doc = read_repo_file("docs/bijux-dev/gh-workflows/release-validation.md");

    assert!(
        gh_makefile.contains("gh-release-validate: install release-validate-rs"),
        "makes/gh.mk must expose the CI entrypoint declared by the release validation suite"
    );
    assert!(
        rust_makefile.contains("[patch.crates-io]"),
        "release validation make support must patch crates-io inside the clean release tree for staged DAG publish verification"
    );
    assert!(
        workflow.contains("name: release-validation"),
        "release validation workflow must declare the canonical workflow name"
    );
    assert!(
        workflow.contains("make gh-release-validate"),
        "release validation workflow must execute the CI make entrypoint"
    );
    assert!(
        workflow.contains("branches:\n      - main"),
        "release validation workflow must validate pushes to main"
    );
    assert!(
        workflow.contains("pull_request:"),
        "release validation workflow must validate pull requests"
    );
    assert!(
        operations_doc.contains("make release-validate-rs"),
        "release validation operations doc must name the canonical local entrypoint"
    );
    assert!(
        operations_doc.contains("make gh-release-validate"),
        "release validation operations doc must name the canonical CI entrypoint"
    );
    assert!(
        operations_doc.contains("cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify"),
        "release validation operations doc must name the maintainer command surface"
    );
    assert!(
        workflow_doc.contains("make gh-release-validate"),
        "release validation workflow doc must name the canonical CI entrypoint"
    );
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__release_validation_suite_explain_output_matches_governed_contract() {
    let suite = read_suite();
    let explain = run_release_explain_verify();
    let explain_data = &explain["data"];

    assert_eq!(explain["command"], "release.explain");
    assert_eq!(explain["status"], "ok");
    assert_eq!(explain_data["command_surface"]["local_entrypoint"], suite.local_entrypoint);
    assert_eq!(explain_data["command_surface"]["ci_entrypoint"], suite.ci_entrypoint);
    assert_eq!(
        explain_data["command_surface"]["maintainer_entrypoint"],
        suite.maintainer_entrypoint
    );
    assert_eq!(explain_data["docs"]["operator_handbook"], suite.documentation.operator_handbook);
    assert_eq!(explain_data["docs"]["workflow_handbook"], suite.documentation.workflow_handbook);
    assert_eq!(explain_data["docs"]["release_operations"], suite.documentation.release_operations);
    assert_eq!(explain_data["package_boundary_contract"], suite.package_boundary_contract);
    assert_eq!(explain_data["flow"], serde_json::to_value(&suite.verify_flow).expect("flow value"));
    assert_eq!(
        explain_data["commands"],
        serde_json::to_value(&suite.commands).expect("commands value")
    );
    assert_eq!(
        explain_data["public_dag_crates"],
        serde_json::to_value(&suite.public_dag_crates).expect("crate value")
    );
    assert_eq!(
        explain_data["artifacts"],
        serde_json::to_value(&suite.artifacts).expect("artifact value")
    );
    assert_eq!(
        explain_data["failure_ownership"],
        serde_json::to_value(&suite.failure_ownership).expect("failure value")
    );
}
