use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> serde_json::Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/benchmark_signal_governance.json"))
            .expect("read benchmark signal governance policy"),
    )
    .expect("parse benchmark signal governance policy")
}

#[test]
fn benchmark_governance_policy_declares_claim_gate_and_noise_rules() {
    let policy = policy();
    assert_eq!(
        policy["governance_rules"]["each_benchmark_declares_supported_claim"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["each_benchmark_declares_gate_class"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["each_benchmark_declares_noise_class"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["benchmark_docs_must_reference_generated_outputs_only"],
        true
    );
}

#[test]
fn benchmark_scenarios_cover_claim_families_and_thresholds() {
    let policy = policy();
    let claim_families: BTreeSet<String> = policy["claim_families"]
        .as_array()
        .expect("claim_families array")
        .iter()
        .map(|entry| entry.as_str().expect("claim family").to_string())
        .collect();
    let mut covered_claims = BTreeSet::new();

    for scenario in policy["scenarios"].as_array().expect("scenarios array") {
        let claim = scenario["supported_claim"]
            .as_str()
            .expect("supported_claim");
        let gate_class = scenario["gate_class"].as_str().expect("gate_class");
        let noise_class = scenario["noise_class"].as_str().expect("noise_class");
        let threshold = scenario["threshold_assertion"]
            .as_str()
            .expect("threshold_assertion");
        let source = scenario["source_report"].as_str().expect("source_report");

        assert!(
            claim_families.contains(claim),
            "unknown claim family: {claim}"
        );
        assert!(
            ["gating", "advisory"].contains(&gate_class),
            "invalid gate class for claim {claim}: {gate_class}"
        );
        assert!(
            ["low", "medium", "high"].contains(&noise_class),
            "invalid noise class for claim {claim}: {noise_class}"
        );
        assert!(
            root().join(threshold).exists(),
            "missing threshold assertion file: {threshold}"
        );
        assert!(
            root().join(source).exists(),
            "missing source benchmark report: {source}"
        );
        covered_claims.insert(claim.to_string());
    }

    assert_eq!(
        covered_claims, claim_families,
        "every claim family must have at least one scenario"
    );
}

#[test]
fn benchmark_signal_quality_reports_exist() {
    for rel in [
        "docs/reports/foundation/benchmark_scenarios_by_claim_report.md",
        "docs/reports/foundation/benchmark_scenarios_without_release_claim_report.md",
        "docs/reports/foundation/release_claims_without_benchmark_scenario_report.md",
        "docs/reports/foundation/flaky_noisy_benchmark_report.md",
        "docs/reports/foundation/slow_benchmark_signal_value_report.md",
        "docs/reports/foundation/benchmark_advisory_to_gating_candidates_report.md",
        "docs/reports/foundation/benchmark_gating_to_advisory_candidates_report.md",
        "docs/reports/foundation/benchmark_trend_by_claim_family_report.md",
        "docs/reports/foundation/benchmark_gaps_by_roadmap_pillar_report.md",
        "docs/reports/foundation/benchmark_threshold_assertions_runtime_helpers.json",
        "docs/reports/foundation/benchmark_docs_generated_sources_guard.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing benchmark quality artifact: {rel}"
        );
    }
}

#[test]
fn benchmark_review_checklist_covers_claim_gate_and_noise() {
    let checklist = fs::read_to_string(root().join("docs/reference/BENCHMARK_REVIEW_CHECKLIST.md"))
        .expect("read benchmark review checklist");
    for token in [
        "supported claim",
        "gate class",
        "noise class",
        "Raw data path",
    ] {
        assert!(
            checklist.contains(token),
            "benchmark review checklist missing token: {token}"
        );
    }
}

#[test]
fn benchmark_roadmap_pillars_map_to_claims() {
    let policy = policy();
    let claim_families: BTreeSet<String> = policy["claim_families"]
        .as_array()
        .expect("claim_families array")
        .iter()
        .map(|entry| entry.as_str().expect("claim family").to_string())
        .collect();

    let mut covered_claims = BTreeSet::new();
    let pillars = policy["roadmap_pillars"]
        .as_object()
        .expect("roadmap_pillars object");
    for (pillar, claims) in pillars {
        let claims = claims.as_array().expect("pillar claims array");
        assert!(
            !claims.is_empty(),
            "roadmap pillar cannot be empty: {pillar}"
        );
        for claim in claims {
            let claim = claim.as_str().expect("pillar claim family");
            assert!(
                claim_families.contains(claim),
                "unknown claim family in roadmap pillar {pillar}: {claim}"
            );
            covered_claims.insert(claim.to_string());
        }
    }

    assert_eq!(
        covered_claims, claim_families,
        "roadmap pillars must cover all governed claim families"
    );
}

#[test]
fn benchmark_claim_map_matches_report() {
    let policy = policy();
    let report = fs::read_to_string(
        root().join("docs/reports/foundation/benchmark_scenarios_by_claim_report.md"),
    )
    .expect("read scenarios by claim report");

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for scenario in policy["scenarios"].as_array().expect("scenarios array") {
        let scenario_id = scenario["scenario_id"]
            .as_str()
            .expect("scenario_id")
            .to_string();
        let claim = scenario["supported_claim"]
            .as_str()
            .expect("supported_claim")
            .to_string();
        grouped.entry(claim).or_default().push(scenario_id);
    }

    for (claim, scenarios) in grouped {
        assert!(
            report.contains(&format!("`{claim}`")),
            "claim missing from report: {claim}"
        );
        for scenario in scenarios {
            assert!(
                report.contains(&format!("`{scenario}`")),
                "scenario missing from claim report: {scenario}"
            );
        }
    }
}
