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
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_subset(root: &std::path::Path, rel: &str) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(root.join(rel)).expect("read subset config"))
        .expect("parse subset config")
}

#[test]
fn battle_and_release_subsets_exist_and_are_trust_focused() {
    let root = repo_root();
    let battle = load_subset(&root, "configs/suites/battle_first.json");
    let release = load_subset(&root, "configs/suites/release_first.json");

    assert_eq!(battle["id"], "battle-first");
    assert_eq!(release["id"], "release-first");

    for (name, cfg) in [("battle-first", battle), ("release-first", release)] {
        let commands = cfg["commands"]
            .as_array()
            .expect("subset commands should be array");
        assert!(!commands.is_empty(), "subset must declare commands: {name}");
        let focus = cfg["report_focus"]
            .as_str()
            .expect("subset report_focus should be string");
        assert!(
            focus.contains("trust") || focus.contains("proof"),
            "subset report_focus must be trust/proof centered: {name}"
        );
    }
}

#[test]
fn evidence_workflow_runs_battle_first_and_release_first_subsets() {
    let root = repo_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/evidence-verify.yml"))
        .expect("read evidence workflow");

    for required in [
        "run battle-first trust subset",
        "run release-first trust subset",
        "verify evidence-battle",
        "verify evidence-cache",
        "verify evidence-replay",
        "verify evidence-consumers",
        "verify evidence-release-set",
        "repo release-evidence-report",
    ] {
        assert!(
            workflow.contains(required),
            "evidence workflow missing trust subset token: {required}"
        );
    }
}

#[test]
fn trust_reports_do_not_use_raw_test_count_bragging() {
    let root = repo_root();
    for rel in [
        "evidence/reports/evidence_verification_summary.md",
        "docs/reports/foundation/release_evidence_report.md",
        "docs/reports/foundation/trust_property_to_test_report.md",
    ] {
        let report = fs::read_to_string(root.join(rel)).expect("read trust report");
        assert!(
            !report.contains("tests passed"),
            "trust report must avoid raw pass-count language: {rel}"
        );
        assert!(
            report.contains("trust") || report.contains("proof"),
            "trust report must mention trust or proof focus: {rel}"
        );
    }
}
