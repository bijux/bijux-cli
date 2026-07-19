use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::path::Path;

mod support;

#[test]
fn replay_fixture_family_exists() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = if repo.join("evidence/cache/replay").is_dir() {
        repo.join("evidence/cache/replay")
    } else {
        repo.join("evidence/dag/cache/replay")
    };
    for rel in [
        "match_case.json",
        "mismatch_case.json",
        "corruption_case.json",
        "unsupported_version_case.json",
        "cache_hit_case.json",
        "cache_miss_case.json",
        "missing_artifact_case.json",
        "incompatible_backend_case.json",
        "regression_corpus.json",
    ] {
        assert!(root.join(rel).exists(), "missing replay fixture: {}", rel);
    }
}

#[test]
fn replay_battle_scenario_declares_mandatory_proof() {
    let value = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/battle/workflows/replay/replay_semantic_comparison.json",
    );
    let assertions = value["assertions"].as_array().expect("assertions array");
    assert!(assertions.iter().any(|v| v == "replay_mandatory_proof"));
}

#[test]
fn replay_cache_and_backend_error_fixtures_are_semantically_typed() {
    let cache_hit = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/cache_hit_case.json",
    );
    assert_eq!(cache_hit["expect"], "equivalent");
    assert_eq!(cache_hit["cache_result"], "hit");

    let cache_miss = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/cache_miss_case.json",
    );
    assert_eq!(cache_miss["expect"], "not_equivalent");
    assert_eq!(cache_miss["cache_result"], "miss");
    assert_eq!(cache_miss["cause_group"], "artifact_payload");

    let missing_artifact = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/missing_artifact_case.json",
    );
    assert_eq!(missing_artifact["expect"], "verification_error");

    let incompatible_backend = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/incompatible_backend_case.json",
    );
    assert_eq!(incompatible_backend["expect"], "incompatible_backend_error");
}

#[test]
fn replay_regression_corpus_covers_core_replay_paths() {
    let corpus = support::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/regression_corpus.json",
    );
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases array");
    assert!(cases.len() >= 6, "expected replay regression breadth");
    for coverage_key in [
        "determinism",
        "partial_graph",
        "imported_run",
        "corrupted_artifact",
        "missing_artifact",
        "backend_compatibility",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"].as_array().is_some_and(|cov| cov.iter().any(|v| v == coverage_key))
            }),
            "replay regression corpus missing coverage: {coverage_key}"
        );
    }
}
