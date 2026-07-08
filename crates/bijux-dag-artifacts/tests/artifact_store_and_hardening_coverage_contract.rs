use bijux_dag_artifacts::store::{
    ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore, ObjectArtifactStore,
};
use bijux_dag_artifacts::{
    build_cleanup_plan, verify_run_dir, write_json_atomic_durable, VerificationMode,
};
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::fs;
use thiserror as _;

#[test]
fn artifact_store_backends_are_exercised_for_coverage_gate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let fs_store = FilesystemArtifactStore::new(dir.path());
    fs_store.write_bytes("cas/x/payload", b"payload").expect("write payload");
    let loaded = fs_store.read_bytes("cas/x/payload").expect("read payload");
    assert_eq!(loaded, b"payload");

    let object = ObjectArtifactStore { bucket: "b".to_string(), prefix: "p".to_string() };
    let caps = object.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::ModeledOnly);
    assert!(!caps.can_write_bytes);
    assert!(!caps.can_read_bytes);
    assert!(object.write_bytes("k", b"v").is_err());
    assert!(object.read_bytes("k").is_err());
}

#[test]
fn hardening_io_paths_are_exercised_for_coverage_gate() {
    let dir = tempfile::tempdir().expect("tempdir");
    let run_dir = dir.path();

    write_json_atomic_durable(run_dir.join("manifest.json"), &json!({"run_id":"run-1"}))
        .expect("write manifest");
    write_json_atomic_durable(run_dir.join("outputs.index.json"), &json!({"files":[]}))
        .expect("write outputs");
    fs::create_dir_all(run_dir.join("trace")).expect("create trace dir");

    let report = verify_run_dir(run_dir, VerificationMode::Standard).expect("verify run dir");
    assert!(report.valid, "standard verification should pass with required files");

    let plan = build_cleanup_plan(&["runs/run-1".to_string(), "tmp/cache".to_string()], &["runs/"]);
    assert_eq!(plan.retained, vec!["runs/run-1".to_string()]);
    assert_eq!(plan.prunable, vec!["tmp/cache".to_string()]);
}
