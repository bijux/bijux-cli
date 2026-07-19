use bijux_dag_artifacts::fs::node_output_relpath;
use bijux_dag_artifacts::hash::sha256_hex;
use bijux_dag_artifacts::paths::is_normalized_relative_path;
use bijux_dag_artifacts::platform::{
    explain_lineage_safe_gc, lineage_dependencies, lineage_dependents, plan_lineage_safe_gc,
};
use bijux_dag_artifacts::retention::RetentionPolicy;
use bijux_dag_artifacts::store::{
    ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore, ObjectArtifactStore,
};
use bijux_dag_artifacts::{write_outputs_index, DeclaredOutputArtifact, OutputsIndex};
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use thiserror as _;

#[test]
fn local_store_roundtrip_and_typed_capabilities_hold_for_io_store() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = FilesystemArtifactStore::new(dir.path());
    store.write_bytes("cas/aa/blob.bin", b"payload").expect("write");
    let loaded = store.read_bytes("cas/aa/blob.bin").expect("read");
    assert_eq!(loaded, b"payload");

    let caps = store.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::Implemented);
    assert!(caps.can_write_bytes);
    assert!(caps.can_read_bytes);
}

#[test]
fn fs_materialization_rejects_traversal_and_non_normalized_paths() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(dir.path().join("ok.txt"), b"ok").expect("write");

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
                name: "ok".to_string(),
                path: "ok.txt".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
        ],
    )
    .expect_err("path traversal must be rejected");
    assert!(err.to_string().contains("normalized relative path"));

    assert!(!is_normalized_relative_path("../escape.txt"));
    assert!(!is_normalized_relative_path("nested\\win.txt"));
    assert!(is_normalized_relative_path("nested/ok.txt"));
    assert_eq!(node_output_relpath("extract", "a.txt"), "nodes/extract/outputs/a.txt");
}

#[test]
fn duplicate_payload_identity_can_exist_with_distinct_provenance_keys() {
    let digest = sha256_hex(b"same-binary");
    let first = format!("run-1:extract:payload.bin:{digest}");
    let second = format!("run-2:import:payload.bin:{digest}");
    assert_ne!(first, second);
    assert_eq!(sha256_hex(b"same-binary"), digest);
}

#[test]
fn nested_tree_export_style_index_and_empty_payload_identity_are_stable() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::create_dir_all(dir.path().join("nested/deeper")).expect("mkdir");
    fs::write(dir.path().join("nested/deeper/empty.bin"), b"").expect("empty");
    fs::write(dir.path().join("nested/deeper/data.bin"), b"abc").expect("data");

    write_outputs_index(
        dir.path(),
        "pack",
        "fp-pack",
        &[
            DeclaredOutputArtifact {
                name: "data".to_string(),
                path: "nested/deeper/data.bin".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
            DeclaredOutputArtifact {
                name: "empty".to_string(),
                path: "nested/deeper/empty.bin".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
        ],
    )
    .expect("index");
    let parsed: OutputsIndex =
        serde_json::from_str(&fs::read_to_string(dir.path().join("index.json")).expect("read"))
            .expect("parse");
    assert_eq!(parsed.files[0].path, "nested/deeper/data.bin");
    assert_eq!(parsed.files[0].size_bytes, 3);
    assert_eq!(parsed.files[1].size_bytes, 0);
    assert_eq!(parsed.files[1].sha256, sha256_hex(b""));
}

#[test]
fn lineage_queries_and_semantic_ordering_remain_stable_on_repeat() {
    let snapshot = bijux_dag_artifacts::lineage::ArtifactLineageSnapshot {
        schema_version: "lineage/v1".to_string(),
        edges: vec![
            bijux_dag_artifacts::lineage::ArtifactLineageEdge {
                artifact_id: "prep:clean.csv".to_string(),
                producer_node_id: "prep".to_string(),
                upstream_artifact_ids: vec!["extract:raw.csv".to_string()],
            },
            bijux_dag_artifacts::lineage::ArtifactLineageEdge {
                artifact_id: "train:model.bin".to_string(),
                producer_node_id: "train".to_string(),
                upstream_artifact_ids: vec!["prep:clean.csv".to_string()],
            },
        ],
    };

    let deps_a = lineage_dependencies(&snapshot, "train:model.bin");
    let deps_b = lineage_dependencies(&snapshot, "train:model.bin");
    assert_eq!(deps_a, deps_b);

    let down_a = lineage_dependents(&snapshot, "prep:clean.csv");
    let down_b = lineage_dependents(&snapshot, "prep:clean.csv");
    assert_eq!(down_a, down_b);
    assert_eq!(down_a, vec!["train:model.bin".to_string()]);
}

#[test]
fn retention_and_gc_explain_decisions_cover_retained_and_collectable_sets() {
    let referenced = vec![
        bijux_dag_artifacts::index::ArtifactId("root:keep.bin".to_string()),
        bijux_dag_artifacts::index::ArtifactId("prep:keep.csv".to_string()),
    ];
    let all = vec![
        bijux_dag_artifacts::index::ArtifactId("root:keep.bin".to_string()),
        bijux_dag_artifacts::index::ArtifactId("prep:keep.csv".to_string()),
        bijux_dag_artifacts::index::ArtifactId("tmp:drop.log".to_string()),
    ];

    let plan = plan_lineage_safe_gc(&referenced, &all, "lineage-io-1");
    assert!(plan.preserved_artifacts.iter().any(|id| id.0 == "root:keep.bin"));
    assert!(plan.collectable_artifacts.iter().any(|id| id.0 == "tmp:drop.log"));

    let explain = explain_lineage_safe_gc(&referenced, &all, "lineage-io-1");
    assert!(explain
        .entries
        .iter()
        .any(|entry| { entry.artifact_id.0 == "root:keep.bin" && entry.action == "preserve" }));
    assert!(explain
        .entries
        .iter()
        .any(|entry| { entry.artifact_id.0 == "tmp:drop.log" && entry.action == "collect" }));

    let retention = RetentionPolicy::default();
    assert_eq!(retention.run_artifacts_ttl_days, 30);
    assert!(retention.retain_prefixes().contains(&"run-"));
}

#[test]
fn content_address_identity_is_stable_for_binary_empty_and_large_streaming_payloads() {
    let binary = [0_u8, 159, 250, 1, 2, 3, 254, 255];
    let binary_hash = sha256_hex(&binary);
    assert_eq!(binary_hash, sha256_hex(&binary));

    let empty_hash = sha256_hex(b"");
    assert_eq!(empty_hash, sha256_hex(b""));

    let mut large = Vec::with_capacity(2 * 1024 * 1024);
    for i in 0..(2 * 1024 * 1024) {
        large.push((i % 251) as u8);
    }
    let full = sha256_hex(&large);

    let mut streamed = Vec::new();
    for chunk in large.chunks(64 * 1024) {
        streamed.extend_from_slice(chunk);
    }
    assert_eq!(full, sha256_hex(&streamed));
}

#[test]
fn modeled_object_store_rejects_io_without_silent_partial_state() {
    let store = ObjectArtifactStore { bucket: "model".to_string(), prefix: "artifact".to_string() };
    assert!(store.write_bytes("x", b"payload").is_err());
    assert!(store.read_bytes("x").is_err());
}

#[test]
fn newline_variants_produce_distinct_content_identity_when_bytes_differ() {
    let lf = sha256_hex("line1\nline2\n".as_bytes());
    let crlf = sha256_hex("line1\r\nline2\r\n".as_bytes());
    assert_ne!(lf, crlf);
}
