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
fn benchmark_scenarios_are_owned_versioned_and_documented() {
    let root = repo_root();
    let required = [
        "evidence/perf/scenarios/tiny_canonical.json",
        "evidence/perf/scenarios/wide_canonical.json",
        "evidence/perf/scenarios/deep_canonical.json",
        "evidence/perf/scenarios/tenk_nodes_canonical.json",
        "evidence/perf/scenarios/large_artifact_canonical.json",
        "evidence/perf/scenarios/cache_heavy_canonical.json",
        "evidence/perf/scenarios/failure_injection_canonical.json",
        "evidence/perf/scenarios/replay_canonical.json",
        "evidence/perf/scenarios/diff_canonical.json",
        "evidence/perf/scenarios/portability_canonical.json",
        "evidence/perf/scenarios/determinism_score.json",
        "evidence/perf/scenarios/replay_fidelity_score.json",
        "evidence/perf/scenarios/explainability_quality.json",
        "evidence/perf/scenarios/artifact_lineage_completeness.json",
        "evidence/perf/scenarios/portability_success_rate.json",
    ];
    for rel in required {
        let path = root.join(rel);
        let payload = fs::read_to_string(&path).expect("read scenario file");
        let value: Value = serde_json::from_str(&payload).expect("parse scenario json");
        assert!(
            value.get("scenario_id").and_then(Value::as_str).is_some(),
            "scenario_id missing in {}",
            path.display()
        );
        assert!(
            value.get("version").and_then(Value::as_str).is_some(),
            "version missing in {}",
            path.display()
        );
        assert!(
            value.get("owner").and_then(Value::as_str).is_some(),
            "owner missing in {}",
            path.display()
        );
    }

    let contract = fs::read_to_string(root.join("docs/spec/PERFORMANCE_CONTRACT.md"))
        .expect("read performance contract");
    for token in [
        "tiny_canonical.json",
        "wide_canonical.json",
        "deep_canonical.json",
        "tenk_nodes_canonical.json",
        "large_artifact_canonical.json",
        "cache_heavy_canonical.json",
        "failure_injection_canonical.json",
        "replay_canonical.json",
        "diff_canonical.json",
        "portability_canonical.json",
    ] {
        assert!(
            contract.contains(token),
            "performance contract missing canonical scenario reference `{}`",
            token
        );
    }

    for contract in [
        "docs/spec/BENCHMARK_SCENARIO_CONTRACT.md",
        "docs/spec/BENCHMARK_REPRODUCIBILITY_CONTRACT.md",
        "docs/spec/COMPARISON_METHOD_CONTRACT.md",
        "docs/spec/EVIDENCE_PUBLICATION_CONTRACT.md",
        "evidence/perf/scenario_registry.json",
    ] {
        assert!(
            root.join(contract).exists(),
            "benchmark contract surface missing: {contract}"
        );
    }
}
