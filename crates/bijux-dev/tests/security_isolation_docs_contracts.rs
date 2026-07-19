use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn read_repo_file(path: &str) -> String {
    let absolute = workspace_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn security_truth_reference_covers_current_runtime_boundaries() {
    let truth = read_repo_file("docs/bijux-dag/operations/security-isolation-truth.md");

    for token in [
        "shell backend",
        "container backend",
        "clean environment",
        "Network policy",
        "Clock policy",
        "Filesystem boundaries",
        "What is enforced",
        "What is best-effort",
        "What is not protected",
        "declared output target preflight",
        "../escape.txt",
        "symlinked existing parent components",
        "there is no socket firewall for shell subprocesses",
        "this is not a VM boundary",
        "replay `--sandbox`",
        "crates/bijux-dag-runtime/tests/policy_cache_contract.rs",
        "crates/bijux-dag-runtime/tests/subprocess_cleanup_contracts.rs",
        "crates/bijux-dag-app/tests/policy_enforcement_surface_contract.rs",
    ] {
        assert!(truth.contains(token), "security truth page missing token: {token}");
    }
}

#[test]
fn operations_docs_route_security_questions_to_truth_reference() {
    for path in [
        "docs/bijux-dag/operations/index.md",
        "docs/bijux-dag/operations/security-and-safety.md",
        "docs/bijux-dag/operations/trust-boundaries.md",
        "docs/bijux-dag/operations/deployment-boundaries.md",
        "docs/bijux-dag/operations/first-run-tutorial.md",
        "docs/bijux-dag/index.md",
        "README.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("security-isolation-truth.md")
                || content.contains("Security And Isolation Truth"),
            "{path} must route security boundary questions to the truth reference"
        );
    }
}

#[test]
fn backend_and_storage_specs_keep_boundary_language_honest() {
    let backend = read_repo_file("docs/spec/BACKEND_CONTRACT.md");
    let storage = read_repo_file("docs/spec/STORAGE_CONTRACT.md");

    for token in [
        "Boundary truth",
        "not a host sandbox",
        "not documented as a VM boundary",
        "deny flags gate declared effects before execution starts",
        "declared output targets must be authorized before backend launch",
        "symlinked existing parent components",
    ] {
        assert!(backend.contains(token), "backend contract missing token: {token}");
    }

    for token in [
        "outputs/index.json",
        "declared output targets must be validated before execution starts",
        "rooted input and output authorization must reject paths that escape",
        "reject `../escape`, absolute paths,",
        "symlinked existing parent escapes",
        "host filesystem sandbox for shell execution",
        "cache metadata must include the retained cache key",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
    ] {
        assert!(storage.contains(token), "storage contract missing token: {token}");
    }

    assert!(
        !storage.contains("outputs.index.json"),
        "storage contract must use the live outputs/index.json path"
    );
}
