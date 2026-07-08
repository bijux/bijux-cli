use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[test]
fn backend_conformance_fixture_has_expected_shape() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/perf/fixtures/infrastructure/backend_conformance_matrix.json");
    let payload = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
    let value: serde_json::Value = serde_json::from_str(&payload)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", fixture_path.display()));
    assert_eq!(value.get("schema_version").and_then(|v| v.as_str()), Some("v0.1"));
    let backends =
        value.get("backends").and_then(|v| v.as_array()).expect("backends array should exist");
    assert!(!backends.is_empty());
    for backend in backends {
        assert!(backend.get("backend").and_then(|v| v.as_str()).is_some());
        assert!(backend.get("checks").and_then(|v| v.as_array()).is_some());
    }
}
