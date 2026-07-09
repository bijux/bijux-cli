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
fn evidence_audit_review_reports_exist() {
    let root = repo_root();
    for required in [
        "evidence/reports/evidence_audit_2026-03-07.md",
        "evidence/reports/evidence_topology_before_after.md",
        "evidence/reports/evidence_root_consolidation_report.md",
        "evidence/reports/release_evidence_strength_before_after.md",
        "evidence/reports/evidence_architecture_freeze_review_cycle.md",
        "evidence/reports/evidence_roast_memo_2026-03-07.md",
    ] {
        assert!(root.join(required).exists(), "missing evidence audit review report: {required}");
    }
}

#[test]
fn evidence_audit_report_covers_counts_and_strength_lists() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("evidence/reports/evidence_audit_2026-03-07.md"))
        .expect("read evidence audit report");
    for token in [
        "All evidence assets by family",
        "Release-blocking assets by family",
        "Advisory assets by family",
        "Strongest evidence assets (top 20)",
        "Weakest evidence assets still present (top 20)",
        "Deletions in this audit wave",
    ] {
        assert!(text.contains(token), "evidence audit report missing token: {token}");
    }
}

#[test]
fn roast_memo_is_honest_about_shallow_and_fraudulent_patterns() {
    let root = repo_root();
    let text = fs::read_to_string(root.join("evidence/reports/evidence_roast_memo_2026-03-07.md"))
        .expect("read evidence roast memo");
    for token in [
        "What is still shallow",
        "What would be fraudulent",
        "Required next hardening moves",
        "advisory compare/perf assets",
    ] {
        assert!(text.contains(token), "roast memo missing required honesty token: {token}");
    }
}

#[test]
fn stale_shallow_audit_and_speculative_report_are_deleted() {
    let root = repo_root();
    for deleted in [
        "evidence/audit/shallow_evidence_audit_2026-03-07.md",
        "evidence/reports/speculative_assets_report.md",
    ] {
        assert!(
            !root.join(deleted).exists(),
            "stale weak evidence report should be deleted: {deleted}"
        );
    }
}
