use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn k8s_adapter_contract_doc_exists_with_required_semantics() {
    let root = repo_root();
    let path = root.join("docs/spec/K8S_ADAPTER_CONTRACT.md");
    assert!(path.exists(), "missing k8s adapter contract doc");
    let text = fs::read_to_string(path).expect("read k8s contract");

    for token in [
        "Resource mapping",
        "Timeout mapping",
        "Retry mapping",
        "Cancellation mapping",
        "Failure normalization",
        "Secret/config injection",
        "Workdir volume semantics",
        "Async watch event handling",
        "Supported and out-of-scope surfaces",
        "Intentionally rejected approximations",
    ] {
        assert!(
            text.contains(token),
            "k8s contract doc missing required section: {token}"
        );
    }
}

#[test]
fn k8s_runtime_contract_tests_cover_equivalence_and_event_determinism() {
    let root = repo_root();
    let test_source = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs"),
    )
    .expect("read backend cluster contracts");

    for token in [
        "simple",
        "fan-out",
        "fan-in",
        "cache-hit",
        "partial-replay",
        "K8S_POD_EVICTED",
        "K8S_IMAGE_PULL_BACKOFF",
        "K8S_POD_PENDING_TIMEOUT",
        "validate_k8s_injection",
        "canonical_k8s_terminal_events",
        "reconcile_k8s_watch_stream",
        "k8s_capability_declaration",
        "reject_unsupported_k8s_fields",
    ] {
        assert!(
            test_source.contains(token),
            "backend cluster contract tests missing token: {token}"
        );
    }
}
