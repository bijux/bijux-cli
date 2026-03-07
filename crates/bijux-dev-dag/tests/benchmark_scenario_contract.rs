use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
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
        "benchmarks/scenarios/tiny_canonical.json",
        "benchmarks/scenarios/medium_canonical.json",
        "benchmarks/scenarios/wide_canonical.json",
        "benchmarks/scenarios/deep_canonical.json",
        "benchmarks/scenarios/cache_heavy_canonical.json",
        "benchmarks/scenarios/replay_canonical.json",
        "benchmarks/scenarios/many_small_nodes_scheduler_overhead.json",
        "benchmarks/scenarios/manifest_trace_write_amplification.json",
        "benchmarks/scenarios/replay_verification_cost.json",
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
        "medium_canonical.json",
        "wide_canonical.json",
        "deep_canonical.json",
        "cache_heavy_canonical.json",
        "replay_canonical.json",
    ] {
        assert!(
            contract.contains(token),
            "performance contract missing canonical scenario reference `{}`",
            token
        );
    }
}
