use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn benchmark_501_520_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/BENCHMARK_MINIMALISM_POLICY.md",
        "docs/reports/foundation/benchmark_redundancy_501_520_report.md",
        "docs/reports/foundation/benchmark_compact_pack_graph_identity_report.md",
        "docs/reports/foundation/benchmark_compact_pack_replay_diff_proof_report.md",
        "docs/reports/foundation/benchmark_compact_pack_artifact_store_report.md",
        "docs/reports/foundation/benchmark_compact_pack_history_operator_report.md",
        "docs/reports/foundation/benchmark_compact_pack_runtime_scheduler_report.md",
        "docs/reports/foundation/benchmark_minimal_set_report.md",
        "docs/reports/foundation/benchmark_retirement_candidate_report.md",
        "docs/reports/foundation/benchmark_signal_to_cost_report.md",
        "docs/reports/foundation/benchmark_health_dashboard.md",
        "docs/reports/foundation/benchmark_review_checklist.md",
        "docs/reports/foundation/benchmark_minimalism_501_520_status_report.md",
        "configs/suites/benchmark_minimalism_verification.json",
        "docs/adr/20260308-benchmark-minimalism.md",
    ] {
        assert!(root.join(rel).exists(), "missing benchmark artifact: {rel}");
    }
}

#[test]
fn benchmark_status_report_maps_501_520() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/benchmark_minimalism_501_520_status_report.md"),
    )
    .expect("read benchmark minimalism status report");

    for token in [
        "501-509",
        "510-515",
        "516-520",
        "benchmark_minimalism_verification.json",
        "20260308-benchmark-minimalism.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn benchmark_minimalism_suite_contains_expected_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/benchmark_minimalism_verification.json"))
            .expect("read benchmark minimalism suite"),
    )
    .expect("parse benchmark minimalism suite");

    assert_eq!(suite["id"], "benchmark-minimalism-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "benchmark_minimalism_guarantees_contracts",
        "benchmark_signal_governance_baseline_contracts",
        "benchmark_signal_quality_contracts",
        "benchmark_signal_reports_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}

#[test]
fn benchmark_signal_policy_keeps_claim_gate_noise_requirements() {
    let root = repo_root();
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/benchmark_signal_governance.json"))
            .expect("read benchmark signal policy"),
    )
    .expect("parse benchmark signal policy");

    let rules = policy["governance_rules"]
        .as_object()
        .expect("governance_rules object");
    for key in [
        "each_benchmark_declares_supported_claim",
        "each_benchmark_declares_gate_class",
        "each_benchmark_declares_noise_class",
        "benchmark_docs_must_reference_generated_outputs_only",
    ] {
        assert_eq!(rules.get(key).and_then(Value::as_bool), Some(true));
    }
}
