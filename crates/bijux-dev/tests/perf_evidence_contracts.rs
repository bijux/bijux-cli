use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_perf_metadata(root: &PathBuf) -> Value {
    serde_json::from_str(
        &fs::read_to_string(root.join("evidence/perf/metadata.json")).expect("read perf metadata"),
    )
    .expect("parse perf metadata")
}

#[test]
fn perf_metadata_contract_reference_resolves_and_matches_scenarios() {
    let root = repo_root();
    let metadata = load_perf_metadata(&root);
    let contract_reference = metadata["contract_reference"].as_str().expect("contract_reference");
    assert!(
        root.join(contract_reference).exists(),
        "perf metadata contract reference must resolve: {contract_reference}"
    );

    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, entry) in scenarios {
        let scenario_contract = entry["contract_reference"].as_str().expect("scenario contract");
        assert_eq!(
            scenario_contract, contract_reference,
            "scenario contract reference must match top-level perf contract: {path}"
        );
        assert!(
            root.join(scenario_contract).exists(),
            "scenario contract reference must resolve: {path} -> {scenario_contract}"
        );
    }
}

#[test]
fn release_blocking_perf_assets_require_threshold_references() {
    let root = repo_root();
    let metadata = load_perf_metadata(&root);
    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, entry) in scenarios {
        let release_blocking = entry["release_blocking"].as_bool().unwrap_or(false);
        let threshold_reference = entry["threshold_reference"].as_str().unwrap_or("");
        if release_blocking {
            assert!(
                !threshold_reference.trim().is_empty(),
                "release-blocking perf scenario missing threshold reference: {path}"
            );
        }
    }
}

#[test]
fn advisory_and_experimental_perf_assets_are_not_release_evidence() {
    let root = repo_root();
    let metadata = load_perf_metadata(&root);
    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for (path, entry) in scenarios {
        let scenario_class = entry["scenario_class"].as_str().expect("scenario_class");
        let release_blocking = entry["release_blocking"].as_bool().unwrap_or(false);
        if scenario_class == "advisory" || scenario_class == "experimental" {
            assert!(
                !release_blocking,
                "advisory or experimental scenario cannot be release_blocking: {path}"
            );
        }
    }
}

#[test]
fn release_relevant_set_is_small_and_core() {
    let root = repo_root();
    let metadata = load_perf_metadata(&root);
    let release_set = metadata["release_relevant_set"].as_array().expect("release_relevant_set");
    assert!(release_set.len() <= 8, "release_relevant_set should remain small and focused");

    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");
    for item in release_set {
        let path = item.as_str().expect("release set path");
        let entry = scenarios.get(path).expect("release set entry must exist in scenarios");
        assert_eq!(
            entry["scenario_class"].as_str().unwrap_or(""),
            "core",
            "release set scenario must be core: {path}"
        );
    }

    for required in [
        "evidence/perf/scenarios/tiny_canonical.json",
        "evidence/perf/scenarios/medium_canonical.json",
        "evidence/perf/scenarios/wide_scheduler_overhead.json",
        "evidence/perf/scenarios/cache_heavy_canonical.json",
        "evidence/perf/scenarios/replay_verification_cost.json",
        "evidence/perf/scenarios/manifest_trace_write_amplification.json",
    ] {
        assert!(
            release_set.iter().any(|item| item.as_str().is_some_and(|value| value == required)),
            "release_relevant_set missing required canonical scenario: {required}"
        );
    }
}

#[test]
fn perf_commands_and_reports_are_wired() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .expect("read commands source");
    for token in [
        "PerfEvidenceSummary",
        "PerfReleaseSet",
        "repo.perf-evidence-summary",
        "repo.perf-release-set",
    ] {
        assert!(source.contains(token), "missing perf evidence command token: {token}");
    }
    assert!(
        root.join("evidence/reports/perf_obsolete_candidates.md").exists(),
        "missing perf obsolete candidate report"
    );
}

#[test]
fn benchmark_registry_covers_required_scenarios_and_metadata_links() {
    let root = repo_root();
    let metadata = load_perf_metadata(&root);
    let scenarios = metadata["scenarios"].as_object().expect("scenarios object");

    let registry_payload = fs::read_to_string(root.join("evidence/perf/scenario_registry.json"))
        .expect("read scenario registry");
    let registry: Value = serde_json::from_str(&registry_payload).expect("parse scenario registry");
    let entries = registry["entries"].as_array().expect("entries array");

    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for entry in entries {
        let id = entry["id"].as_str().expect("id");
        let path = entry["path"].as_str().expect("path");
        assert!(ids.insert(id.to_string()), "duplicate registry id {id}");
        paths.insert(path.to_string());
        assert!(scenarios.contains_key(path), "registry path missing in perf metadata: {path}");
    }

    for required in [
        "tiny-canonical",
        "wide-canonical",
        "deep-canonical",
        "tenk-nodes-canonical",
        "large-artifact-canonical",
        "cache-heavy-canonical",
        "failure-injection-canonical",
        "replay-canonical",
        "diff-canonical",
        "portability-canonical",
        "determinism-score",
        "replay-fidelity-score",
        "explainability-quality",
        "artifact-lineage-completeness",
        "portability-success-rate",
        "inspect-history-latency",
    ] {
        assert!(ids.contains(required), "registry missing required id: {required}");
    }
}
