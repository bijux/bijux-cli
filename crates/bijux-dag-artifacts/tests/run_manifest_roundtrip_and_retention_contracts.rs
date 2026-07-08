use bijux_dag_artifacts::{build_cleanup_plan, retention::RetentionPolicy, Manifest};
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;
use thiserror as _;

mod support;

#[test]
fn run_manifest_minimal_and_maximal_roundtrip_remain_conformant() {
    for fixture in ["run_manifest_minimal.json", "run_manifest_maximal.json"] {
        let payload = support::load_workspace_fixture_text(
            env!("CARGO_MANIFEST_DIR"),
            &format!("crates/bijux-dag-artifacts/tests/fixtures/{fixture}"),
        );
        let parsed: Manifest = serde_json::from_str(&payload).expect("parse fixture manifest");
        let roundtrip = serde_json::to_string_pretty(&parsed).expect("serialize");
        let reparsed: Manifest = serde_json::from_str(&roundtrip).expect("reparse");

        assert_eq!(parsed.run_id, reparsed.run_id);
        assert_eq!(parsed.manifest_version, reparsed.manifest_version);
        assert_eq!(parsed.graph_fingerprint, reparsed.graph_fingerprint);
        assert_eq!(parsed.status, reparsed.status);
        assert_eq!(parsed.node_counts, reparsed.node_counts);
        assert_eq!(parsed.policy, reparsed.policy);
    }
}

#[test]
fn run_manifest_version_migration_fixtures_classify_supported_and_unsupported_versions() {
    let supported: Value = support::load_workspace_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/compat/run_schema/v0_1_supported/minimal.manifest.json",
    );
    let unsupported_past: Value = support::load_workspace_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/compat/run_schema/unsupported_past/minimal.manifest.json",
    );
    let unsupported_future: Value = support::load_workspace_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/compat/run_schema/unsupported_future/minimal.manifest.json",
    );

    assert_eq!(supported["manifest_version"], "run-manifest/v0.1");
    assert_ne!(supported["manifest_version"], unsupported_past["manifest_version"]);
    assert_ne!(supported["manifest_version"], unsupported_future["manifest_version"]);
}

#[test]
fn retention_policy_and_cleanup_plan_match_real_run_layout_prefixes() {
    let policy = RetentionPolicy::default();
    assert!(!policy.should_prune_run_days(policy.run_artifacts_ttl_days));
    assert!(policy.should_prune_run_days(policy.run_artifacts_ttl_days + 1));
    assert!(!policy.should_prune_cache_days(policy.local_cache_ttl_days));
    assert!(policy.should_prune_cache_days(policy.local_cache_ttl_days + 1));

    let entries = vec![
        "run-2026-03-01".to_string(),
        "run-2026-01-01".to_string(),
        "cache-a1".to_string(),
        "promoted-model-v2".to_string(),
        "export-bundle-7".to_string(),
        "tmp-upload".to_string(),
        "scratch".to_string(),
    ];
    let retain = policy.retain_prefixes();
    let plan = build_cleanup_plan(&entries, &retain);

    assert!(plan.retained.iter().any(|e| e.starts_with("run-")));
    assert!(plan.retained.iter().any(|e| e.starts_with("cache-")));
    assert!(plan.retained.iter().any(|e| e.starts_with("promoted-")));
    assert!(plan.retained.iter().any(|e| e.starts_with("export-")));
    assert!(plan.prunable.iter().any(|e| e == "tmp-upload"));
    assert!(plan.prunable.iter().any(|e| e == "scratch"));
}
