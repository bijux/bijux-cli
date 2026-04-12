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

#[test]
fn replay_contract_documents_required_sections() {
    let root = workspace_root();
    let contract = fs::read_to_string(root.join("docs/spec/REPLAY_CONTRACT.md")).expect("contract");

    for section in [
        "## Scope",
        "## Replay definition",
        "## Authoritative inputs",
        "## Replay explain mode",
        "## What replay cannot prove",
        "## Related tests",
        "## Versioning and change policy",
    ] {
        assert!(contract.contains(section), "replay contract missing section: {section}");
    }
}

#[test]
fn replay_hardening_report_links_contract_and_proof_surfaces() {
    let root = workspace_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/REPLAY_HARDENING_REPORT.md"))
            .expect("report");

    for token in [
        "docs/spec/REPLAY_CONTRACT.md",
        "crates/bijux-dag-app/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs",
        "crates/bijux-dev/tests/replay_hardening_contracts.rs",
        "evidence/cache/replay/",
    ] {
        assert!(report.contains(token), "replay hardening report missing: {token}");
    }
}
