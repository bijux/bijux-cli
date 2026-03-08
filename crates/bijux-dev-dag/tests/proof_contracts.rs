use bijux_dag_testkit as _;
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
fn proof_contract_docs_and_schema_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/PROOF_BUNDLE_CONTRACT.md",
        "docs/spec/DETERMINISM_EVIDENCE_CONTRACT.md",
        "docs/spec/INTEGRITY_EVIDENCE_CONTRACT.md",
        "docs/spec/REPLAY_EVIDENCE_CONTRACT.md",
        "docs/spec/AUDIT_REPORT_CONTRACT.md",
        "docs/spec/PROOF_BUNDLE_SCHEMA_v0.1.json",
        "docs/reference/PROOF_REPORT_FORMAT.md",
        "docs/reports/foundation/proof_generation_benchmarks.md",
        "docs/reports/foundation/proof_vs_verification_vs_inspection.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing proof contract surface: {rel}"
        );
    }
}

#[test]
fn cli_and_app_surfaces_include_dag_prove_command() {
    let root = repo_root();
    let cmd_source = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .expect("read command source");
    assert!(
        cmd_source.contains("Prove"),
        "commands must include Prove variant"
    );

    let app_lib =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs")).expect("read app lib");
    let app_routes =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/replay_routes.rs"))
            .expect("read app replay routes");
    for token in ["dag.prove", "build_run_proof_bundle(", "incomplete_reasons"] {
        let present = app_lib.contains(token) || app_routes.contains(token);
        assert!(present, "app prove surface missing token: {token}");
    }
}
