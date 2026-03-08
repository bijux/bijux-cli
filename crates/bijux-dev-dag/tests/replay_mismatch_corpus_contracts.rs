use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

#[test]
fn replay_mismatch_fixture_corpus_is_structured_for_regression_reuse() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let corpus_path = root.join("evidence/cache/replay/mismatch_fixture_corpus.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(corpus_path).expect("corpus")).expect("json");

    assert_eq!(payload["version"], "v1");
    let cases = payload["cases"].as_array().expect("cases");
    assert!(cases.len() >= 5);

    for cause in [
        "manifest_drift",
        "graph_semantics",
        "node_outcomes",
        "artifact_payload",
    ] {
        assert!(
            cases.iter().any(|case| case["cause_group"] == cause),
            "missing cause group fixture: {cause}"
        );
    }
}
