use bijux_dag_artifacts::hardening::{
    build_cleanup_plan, verify_run_dir, write_json_atomic_durable, VerificationMode,
};
use bijux_dag_artifacts::index::ArtifactId;
use bijux_dag_artifacts::platform::{
    explain_lineage_safe_gc, plan_lineage_safe_gc, ArtifactGarbageCollectionExplain,
};
use bijux_dag_artifacts::retention::RetentionPolicy;
use bijux_dag_artifacts::store::{
    ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore, ObjectArtifactStore,
};
use bijux_dag_artifacts::{write_outputs_index, DeclaredOutputArtifact, OutputsIndex};
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use tempfile as _;
use thiserror as _;

#[test]
fn fs_accepts_normalized_relative_paths_for_indexing() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::create_dir_all(dir.path().join("nested/ok")).expect("mkdir");
    fs::write(dir.path().join("nested/ok/payload.bin"), b"payload").expect("write");

    write_outputs_index(
        dir.path(),
        "node-a",
        "fp-a",
        &[DeclaredOutputArtifact {
            name: "payload".to_string(),
            path: "nested/ok/payload.bin".to_string(),
            kind: "file".to_string(),
            media_type: "application/octet-stream".to_string(),
            promotable: false,
        }],
    )
    .expect("index");

    let parsed: OutputsIndex =
        serde_json::from_str(&fs::read_to_string(dir.path().join("index.json")).expect("read"))
            .expect("parse");
    assert_eq!(parsed.files.len(), 1);
    assert_eq!(parsed.files[0].path, "nested/ok/payload.bin");
}

#[test]
fn fs_rejects_escaping_paths() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(dir.path().join("safe.txt"), b"safe").expect("write");

    let err = write_outputs_index(
        dir.path(),
        "node-a",
        "fp-a",
        &[
            DeclaredOutputArtifact {
                name: "escape".to_string(),
                path: "../escape.txt".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
            DeclaredOutputArtifact {
                name: "safe".to_string(),
                path: "safe.txt".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
        ],
    )
    .expect_err("escape must fail");
    assert!(err.to_string().contains("normalized relative path"));
}

#[test]
fn fs_repeated_writes_keep_latest_bytes() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = FilesystemArtifactStore::new(dir.path());

    store.write_bytes("cas/a.bin", b"first").expect("first");
    store.write_bytes("cas/a.bin", b"second").expect("second");

    let loaded = store.read_bytes("cas/a.bin").expect("read");
    assert_eq!(loaded, b"second");
}

#[test]
fn store_local_roundtrip_and_capability_query_are_stable() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = FilesystemArtifactStore::new(dir.path());

    store.write_bytes("cas/aa/blob", b"hello-world").expect("write");
    assert_eq!(store.read_bytes("cas/aa/blob").expect("read"), b"hello-world");

    let caps = store.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::Implemented);
    assert!(caps.can_write_bytes);
    assert!(caps.can_read_bytes);
}

#[test]
fn store_modeled_capability_query_behavior_is_explicit() {
    let store =
        ObjectArtifactStore { bucket: "bucket-a".to_string(), prefix: "prefix-a".to_string() };
    let caps = store.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::ModeledOnly);
    assert!(!caps.can_write_bytes);
    assert!(!caps.can_read_bytes);
    assert!(store.write_bytes("k", b"v").is_err());
    assert!(store.read_bytes("k").is_err());
}

#[test]
fn integrity_verify_flow_flags_missing_required_artifacts() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version": "run-manifest/v0.1",
            "run_id": "run-1"
        }))
        .expect("json"),
    )
    .expect("write");

    let report = verify_run_dir(dir.path(), VerificationMode::Standard).expect("verify");
    assert!(!report.valid);
    assert!(report.anomalies.iter().any(|entry| entry.contains("outputs.index")));
}

#[test]
fn hardening_cleanup_plan_respects_bounded_prefix_policy() {
    let policy = RetentionPolicy::default();
    let entries = vec!["run-1".to_string(), "cache-a".to_string(), "scratch-tmp".to_string()];
    let plan = build_cleanup_plan(&entries, &policy.retain_prefixes());
    assert!(plan.retained.contains(&"run-1".to_string()));
    assert!(plan.retained.contains(&"cache-a".to_string()));
    assert!(plan.prunable.contains(&"scratch-tmp".to_string()));
}

#[test]
fn hardening_manifest_atomicity_replaces_target_content() {
    let dir = tempfile::tempdir().expect("tmp");
    let target = dir.path().join("manifest.json");

    write_json_atomic_durable(&target, &json!({"version": 1, "status": "old"})).expect("v1");
    write_json_atomic_durable(&target, &json!({"version": 2, "status": "new"})).expect("v2");

    let parsed: Value =
        serde_json::from_str(&fs::read_to_string(&target).expect("read")).expect("parse");
    assert_eq!(parsed["version"], 2);
    assert_eq!(parsed["status"], "new");
}

#[test]
fn inspect_lineage_only_records_stay_explainable() {
    let referenced = vec![ArtifactId("lineage:only".to_string())];
    let all = vec![ArtifactId("lineage:only".to_string())];
    let explain = explain_lineage_safe_gc(&referenced, &all, "lineage-only-1");

    assert_eq!(explain.lineage_snapshot_id, "lineage-only-1");
    assert_eq!(explain.entries.len(), 1);
    assert_eq!(explain.entries[0].action, "preserve");
}

#[test]
fn inspect_missing_payload_and_damaged_metadata_are_detected() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({"manifest_version": "run-manifest/v0.1"})).expect("json"),
    )
    .expect("write");

    let strict = verify_run_dir(dir.path(), VerificationMode::Strict).expect("verify");
    assert!(strict.anomalies.iter().any(|entry| entry.contains("missing run_id")));
    assert!(strict.anomalies.iter().any(|entry| entry.contains("manifest.finalized")));
}

#[test]
fn gc_explain_covers_retained_roots_and_collectable_leaves() {
    let referenced = vec![ArtifactId("root:a".to_string())];
    let all = vec![ArtifactId("root:a".to_string()), ArtifactId("leaf:b".to_string())];

    let plan = plan_lineage_safe_gc(&referenced, &all, "lineage-gc-1");
    assert!(plan.preserved_artifacts.iter().any(|id| id.0 == "root:a"));
    assert!(plan.collectable_artifacts.iter().any(|id| id.0 == "leaf:b"));

    let explain: ArtifactGarbageCollectionExplain =
        explain_lineage_safe_gc(&referenced, &all, "lineage-gc-1");
    assert!(explain
        .entries
        .iter()
        .any(|entry| entry.artifact_id.0 == "root:a" && entry.action == "preserve"));
    assert!(explain
        .entries
        .iter()
        .any(|entry| entry.artifact_id.0 == "leaf:b" && entry.action == "collect"));
}

#[test]
fn retention_explain_for_imported_bundle_prefixes_is_stable() {
    let policy = RetentionPolicy::default();
    let entries = vec![
        "export-bundle-2026".to_string(),
        "run-2026-03-08".to_string(),
        "tmp-work".to_string(),
    ];
    let plan = build_cleanup_plan(&entries, &policy.retain_prefixes());
    assert!(plan.retained.iter().any(|entry| entry.starts_with("export-")));
    assert!(plan.prunable.iter().any(|entry| entry == "tmp-work"));
}

#[test]
fn store_capability_serialization_is_stable() {
    let store =
        ObjectArtifactStore { bucket: "cap-bucket".to_string(), prefix: "cap-prefix".to_string() };
    let caps = store.capabilities();

    let first = format!("{caps:?}");
    let second = format!("{:?}", store.capabilities());
    assert_eq!(first, second);
    assert!(first.contains("ModeledOnly"));
}
