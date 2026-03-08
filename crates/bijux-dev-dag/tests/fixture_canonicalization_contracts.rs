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
fn fixture_521_540_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/CANONICAL_FIXTURE_STRATEGY_POLICY.md",
        "docs/reports/foundation/fixture_inventory_521_540_report.md",
        "docs/reports/foundation/fixture_unconsumed_report.md",
        "docs/reports/foundation/fixture_duplicate_report.md",
        "docs/reports/foundation/fixture_split_merge_candidate_report.md",
        "docs/reports/foundation/fixture_orphan_cleanup_report.md",
        "docs/reports/foundation/fixture_compact_index_report.md",
        "docs/reports/foundation/fixture_shrink_report.md",
        "docs/reports/foundation/fixture_usefulness_score_report.md",
        "docs/reports/foundation/fixture_health_dashboard.md",
        "docs/reports/foundation/fixture_contraction_521_540_status_report.md",
        "configs/suites/fixture_contraction_verification.json",
        "docs/adr/20260308-canonical-fixture-strategy.md",
    ] {
        assert!(root.join(rel).exists(), "missing fixture artifact: {rel}");
    }
}

#[test]
fn fixture_governance_policy_includes_required_tags_and_smoke_defaults() {
    let root = repo_root();
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/fixture_family_governance.json"))
            .expect("read fixture governance policy"),
    )
    .expect("parse fixture governance policy");

    let tags = policy["fixture_tags"]
        .as_array()
        .expect("fixture_tags array")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    for required in ["canonical", "stress", "corrupt", "smoke", "legacy"] {
        assert!(tags.contains(&required), "missing fixture tag: {required}");
    }

    let smoke_policy = policy["smoke_default_tag_policy"]
        .as_object()
        .expect("smoke_default_tag_policy object");
    let allowed = smoke_policy["allowed_default_tags"]
        .as_array()
        .expect("allowed_default_tags")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(allowed.contains(&"canonical"));
    assert!(allowed.contains(&"smoke"));

    let forbidden = smoke_policy["forbidden_default_tags"]
        .as_array()
        .expect("forbidden_default_tags")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(forbidden.contains(&"legacy"));
}

#[test]
fn fixture_status_report_maps_521_540_requirements() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/fixture_contraction_521_540_status_report.md"),
    )
    .expect("read fixture contraction status report");

    for token in [
        "521-525",
        "526-534",
        "535-540",
        "fixture_contraction_verification.json",
        "20260308-canonical-fixture-strategy.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn fixture_contraction_suite_contains_expected_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/fixture_contraction_verification.json"))
            .expect("read fixture contraction suite"),
    )
    .expect("parse fixture contraction suite");

    assert_eq!(suite["id"], "fixture-contraction-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "fixture_canonicalization_contracts",
        "fixture_loader_governance_contracts",
        "fixture_tooling_completion_contracts",
        "fixture_helpers_fast_suite_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}
