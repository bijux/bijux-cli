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

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn benchmark_161_180_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/benchmark_signal_governance_161_180_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "161-164 benchmark inventory and gap reports",
        "165-167 benchmark declaration governance rules",
        "168-171 benchmark quality and lane-change reports",
        "172-175 threshold assertions by product claim family",
        "176-177 trend and roadmap-gap reports",
        "178 benchmark review checklist",
        "179 benchmark docs generated-output gate",
        "180 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn benchmark_161_180_governance_artifacts_exist() {
    for rel in [
        "configs/policy/benchmark_signal_governance.json",
        "docs/reports/foundation/benchmark_scenarios_by_claim_report.md",
        "docs/reports/foundation/benchmark_scenarios_without_release_claim_report.md",
        "docs/reports/foundation/release_claims_without_benchmark_scenario_report.md",
        "docs/reports/foundation/benchmarks_without_regression_thresholds_report.md",
        "docs/reports/foundation/flaky_noisy_benchmark_report.md",
        "docs/reports/foundation/slow_benchmark_signal_value_report.md",
        "docs/reports/foundation/benchmark_advisory_to_gating_candidates_report.md",
        "docs/reports/foundation/benchmark_gating_to_advisory_candidates_report.md",
        "docs/reports/foundation/benchmark_threshold_assertions_graph_identity.json",
        "docs/reports/foundation/benchmark_threshold_assertions_run_history.json",
        "docs/reports/foundation/benchmark_threshold_assertions_artifact_trace.json",
        "docs/reports/foundation/benchmark_threshold_assertions_runtime_helpers.json",
        "docs/reports/foundation/benchmark_trend_by_claim_family_report.md",
        "docs/reports/foundation/benchmark_gaps_by_roadmap_pillar_report.md",
        "docs/reference/BENCHMARK_REVIEW_CHECKLIST.md",
        "docs/reports/foundation/benchmark_docs_generated_sources_guard.md",
        "docs/adr/20260308-benchmark-signal-governance.md",
        "crates/bijux-dev-dag/tests/benchmark_signal_quality_contracts.rs",
        "crates/bijux-dev-dag/tests/benchmark_completion_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
