use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn app_e2e_lane_reports_and_suites_exist() {
    for rel in [
        "configs/suites/app_e2e_fast_core.json",
        "configs/suites/app_e2e_slow_extended.json",
        "configs/policy/app_e2e_lane_rationale.json",
        "configs/policy/app_e2e_fast_lane_budget.json",
        "docs/reports/foundation/app_e2e_lane_classification.md",
        "docs/reports/foundation/app_fast_lane_skipped_scenarios_with_reasons.md",
        "docs/reports/foundation/app_promotable_skipped_scenarios.md",
        "docs/reports/foundation/app_slowest_full_lane_scenarios.md",
        "docs/reports/foundation/app_high_value_not_in_fast_lane.md",
        "docs/reports/foundation/app_smoke_release_coverage_report.md",
        "crates/bijux-dag-app/tests/fixtures/git_for_computation_graphs_workflow.json",
        "crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs",
    ] {
        assert!(repo_root().join(rel).exists(), "missing app e2e governance artifact: {rel}");
    }
}

#[test]
fn app_e2e_budget_policy_is_nontrivial() {
    let raw = fs::read_to_string(repo_root().join("configs/policy/app_e2e_fast_lane_budget.json"))
        .expect("read budget policy");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse budget policy");
    assert!(
        value["max_runtime_seconds"].as_u64().unwrap_or_default() >= 60,
        "app e2e fast-lane budget must be at least one minute"
    );
    assert!(
        value["max_scenarios"].as_u64().unwrap_or_default() >= 5,
        "app e2e fast-lane scenario budget must be nontrivial"
    );
}

#[test]
fn every_slow_e2e_test_has_lane_rationale_entry() {
    let e2e_source = fs::read_to_string(
        repo_root().join("crates/bijux-dag-app/tests/e2e_integration_scenarios.rs"),
    )
    .expect("read e2e integration scenarios");
    let mut names = BTreeSet::new();
    for line in e2e_source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fn e2e_") && trimmed.ends_with('{') {
            let token = trimmed
                .trim_start_matches("fn ")
                .trim_end_matches('{')
                .trim();
            let name = token.trim_end_matches("()").to_string();
            names.insert(name);
        }
    }

    let raw = fs::read_to_string(repo_root().join("configs/policy/app_e2e_lane_rationale.json"))
        .expect("read lane rationale policy");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse lane rationale policy");
    let listed: BTreeSet<String> = value["tests"]
        .as_array()
        .expect("tests array")
        .iter()
        .filter_map(|item| item["name"].as_str().map(ToString::to_string))
        .collect();

    for name in names {
        assert!(listed.contains(&name), "missing lane rationale for e2e test: {name}");
    }
}

#[test]
fn smoke_release_coverage_spans_identity_replay_diff_bundle_and_proof() {
    let raw = fs::read_to_string(
        repo_root().join("docs/reports/foundation/app_smoke_release_coverage_report.md"),
    )
    .expect("read smoke release coverage report");
    for token in ["validate", "replay", "diff", "export", "prove"] {
        assert!(raw.contains(token), "smoke release coverage missing token: {token}");
    }
}
