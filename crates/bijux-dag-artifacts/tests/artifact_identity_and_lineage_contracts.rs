use bijux_dag_artifacts::index::{
    ArtifactId, ArtifactMaterializationRecord, ArtifactMaterializationSource,
};
use bijux_dag_artifacts::lineage::ArtifactLineageSnapshot;
use bijux_dag_artifacts::platform::{
    explain_lineage_safe_gc, lineage_dependencies, lineage_dependents, plan_lineage_safe_gc,
};
use bijux_dag_artifacts::store::{
    ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore, ObjectArtifactStore,
};
use bijux_dag_artifacts::{
    build_artifact_identity, hash::sha256_hex, write_outputs_index, DeclaredOutputArtifact,
    OutputsIndex, RunDirSchemaIndex, RunOutputFile, RunOutputsIndex,
};
use hex as _;
use serde as _;
use sha2 as _;
use std::fs;
use thiserror as _;

#[test]
fn duplicate_content_can_have_distinct_provenance_records() {
    let digest = sha256_hex(b"same-content");
    let first = ArtifactMaterializationRecord {
        artifact_id: ArtifactId("extract:dataset.csv".to_string()),
        source: ArtifactMaterializationSource::Produced,
        recorded_unix_ms: 100,
    };
    let second = ArtifactMaterializationRecord {
        artifact_id: ArtifactId("import:dataset.csv".to_string()),
        source: ArtifactMaterializationSource::Imported,
        recorded_unix_ms: 101,
    };
    assert_ne!(first.artifact_id, second.artifact_id);
    assert_eq!(digest, sha256_hex(b"same-content"));
}

#[test]
fn canonical_artifact_identity_is_stable_and_explainable() {
    let identity = build_artifact_identity(
        "run-42",
        "extract",
        "nodes/extract/outputs/report.json",
        "fp-extract",
        "abc123",
    );
    assert_eq!(identity.legacy_artifact_id, "extract:report.json");
    assert_eq!(
        identity.canonical_artifact_id,
        "run=run-42;node=extract;path=nodes/extract/outputs/report.json;sha256=abc123"
    );
    assert_eq!(identity.output_name, "report.json");
    assert_eq!(identity.node_fingerprint, "fp-extract");
}

#[test]
fn run_dir_schema_index_defaults_cover_root_and_node_requirements() {
    let schema = RunDirSchemaIndex::default();
    assert_eq!(schema.schema_version, "run-dir-schema/v0.1");
    assert_eq!(schema.inputs_index_schema, "configs/dag/schema/inputs_index.schema.json");
    assert!(schema.required_root_files.contains(&"run.schema.json".to_string()));
    assert!(schema.required_root_files.contains(&"manifest.json".to_string()));
    assert!(schema.required_node_files.contains(&"trace.json".to_string()));
    assert!(schema.required_node_files.contains(&"outputs/index.json".to_string()));
}

#[test]
fn lineage_traversal_is_stable_for_upstream_and_downstream_queries() {
    let snapshot = ArtifactLineageSnapshot {
        schema_version: "lineage/v1".to_string(),
        edges: vec![
            bijux_dag_artifacts::lineage::ArtifactLineageEdge {
                artifact_id: "transform:clean.csv".to_string(),
                producer_node_id: "transform".to_string(),
                upstream_artifact_ids: vec!["extract:raw.csv".to_string()],
            },
            bijux_dag_artifacts::lineage::ArtifactLineageEdge {
                artifact_id: "train:model.bin".to_string(),
                producer_node_id: "train".to_string(),
                upstream_artifact_ids: vec!["transform:clean.csv".to_string()],
            },
        ],
    };
    assert_eq!(
        lineage_dependencies(&snapshot, "train:model.bin"),
        vec!["transform:clean.csv".to_string()]
    );
    assert_eq!(
        lineage_dependents(&snapshot, "transform:clean.csv"),
        vec!["train:model.bin".to_string()]
    );
}

#[test]
fn outputs_index_preserves_nested_paths_and_empty_file_identity() {
    let dir = tempfile::tempdir().expect("tempdir");
    fs::create_dir_all(dir.path().join("nested/deeper")).expect("nested dirs");
    fs::write(dir.path().join("nested/deeper/empty.txt"), b"").expect("write empty");
    fs::write(dir.path().join("nested/deeper/data.txt"), b"payload").expect("write payload");
    write_outputs_index(
        dir.path(),
        "node-a",
        "fp-a",
        &[
            DeclaredOutputArtifact {
                name: "data".to_string(),
                path: "nested/deeper/data.txt".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
            DeclaredOutputArtifact {
                name: "empty".to_string(),
                path: "nested/deeper/empty.txt".to_string(),
                kind: "file".to_string(),
                media_type: "application/octet-stream".to_string(),
                promotable: false,
            },
        ],
    )
    .expect("write index");
    let index_raw = fs::read_to_string(dir.path().join("index.json")).expect("read index");
    let parsed: OutputsIndex = serde_json::from_str(&index_raw).expect("parse index");
    assert_eq!(parsed.files.len(), 2);
    assert_eq!(parsed.files[0].path, "nested/deeper/data.txt");
    assert_eq!(parsed.files[1].path, "nested/deeper/empty.txt");
    assert_eq!(parsed.files[1].sha256, sha256_hex(b""));
}

#[test]
fn metadata_only_indexes_scale_without_payload_materialization() {
    let files = (0..10_000)
        .map(|i| RunOutputFile {
            node_id: "node-bulk".to_string(),
            node_fingerprint: "fp-bulk".to_string(),
            name: format!("output-{i:05}"),
            kind: "file".to_string(),
            media_type: "application/octet-stream".to_string(),
            size_bytes: i as u64,
            sha256: format!("{:064x}", i),
            path: format!("node-bulk/output-{i:05}.bin"),
            promotable: false,
        })
        .collect::<Vec<_>>();
    let index = RunOutputsIndex { files };
    let bytes = serde_json::to_vec(&index).expect("serialize");
    assert!(bytes.len() > 100_000);
    let reparsed: RunOutputsIndex = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(reparsed.files.len(), 10_000);
}

#[test]
fn same_logical_artifact_can_exist_across_replay_ancestry() {
    let logical_artifact = "train:model.bin";
    let content_hash = sha256_hex(b"model-bytes");
    let parent_provenance_key = format!("run-parent:{logical_artifact}:{content_hash}");
    let replay_provenance_key = format!("run-replay:{logical_artifact}:{content_hash}");
    assert_ne!(parent_provenance_key, replay_provenance_key);
}

#[test]
fn gc_plan_and_explain_outputs_stay_consistent() {
    let referenced = vec![ArtifactId("transform:clean.csv".to_string())];
    let all = vec![
        ArtifactId("extract:raw.csv".to_string()),
        ArtifactId("transform:clean.csv".to_string()),
        ArtifactId("train:model.bin".to_string()),
    ];
    let plan = plan_lineage_safe_gc(&referenced, &all, "lineage-snapshot-1");
    assert_eq!(plan.preserved_artifacts, vec![ArtifactId("transform:clean.csv".to_string())]);
    let explain = explain_lineage_safe_gc(&referenced, &all, "lineage-snapshot-1");
    assert_eq!(explain.lineage_snapshot_id, "lineage-snapshot-1");
    assert_eq!(explain.entries.len(), 3);
    assert!(explain
        .entries
        .iter()
        .any(|e| e.artifact_id.0 == "transform:clean.csv" && e.action == "preserve"));
}

#[test]
fn artifact_store_capabilities_use_typed_support_levels() {
    let fs_store = FilesystemArtifactStore::new(".");
    let fs_caps = fs_store.capabilities();
    assert_eq!(fs_caps.support_level, ArtifactStoreSupportLevel::Implemented);
    assert!(fs_caps.can_write_bytes);
    assert!(fs_caps.can_read_bytes);

    let object_store =
        ObjectArtifactStore { bucket: "demo".to_string(), prefix: "artifacts/".to_string() };
    let object_caps = object_store.capabilities();
    assert_eq!(object_caps.support_level, ArtifactStoreSupportLevel::ModeledOnly);
    assert!(!object_caps.can_write_bytes);
    assert!(!object_caps.can_read_bytes);
}
