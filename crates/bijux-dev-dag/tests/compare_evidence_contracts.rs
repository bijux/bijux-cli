use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn comparison_scenarios_link_to_bijux_executable_evidence() {
    let root = repo_root();
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compare/metadata.json"))
            .expect("read compare metadata"),
    )
    .expect("parse compare metadata");

    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, scenario) in scenarios {
        assert!(
            root.join(path).exists(),
            "comparison scenario file missing from metadata: {path}"
        );
        let bijux_asset = scenario["bijux_evidence_asset"]
            .as_str()
            .expect("bijux_evidence_asset");
        assert!(
            root.join(bijux_asset).exists(),
            "comparison scenario points to missing bijux executable evidence asset: {path} -> {bijux_asset}"
        );
    }
}

#[test]
fn comparison_scenarios_declare_non_equivalence_limits() {
    let root = repo_root();
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compare/metadata.json"))
            .expect("read compare metadata"),
    )
    .expect("parse compare metadata");
    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, scenario) in scenarios {
        let limits = scenario["non_equivalence_limits"]
            .as_array()
            .expect("non_equivalence_limits array");
        assert!(
            !limits.is_empty(),
            "comparison scenario must define non-equivalence limits: {path}"
        );
    }
}

#[test]
fn comparison_scenarios_do_not_claim_unmeasured_release_blocking_parity() {
    let root = repo_root();
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compare/metadata.json"))
            .expect("read compare metadata"),
    )
    .expect("parse compare metadata");
    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, scenario) in scenarios {
        let release_blocking = scenario["release_blocking"].as_bool().unwrap_or(false);
        let measured_bijux_side = scenario["measured_bijux_side"].as_bool().unwrap_or(false);
        if release_blocking {
            assert!(
                measured_bijux_side,
                "comparison scenario cannot be release-blocking without measured bijux side: {path}"
            );
        }
    }
}

#[test]
fn comparison_fact_vs_interpretation_report_exists() {
    let root = repo_root();
    assert!(
        root.join("evidence/reports/comparison_fact_vs_interpretation.md")
            .exists(),
        "comparison fact-vs-interpretation report must exist"
    );
}
