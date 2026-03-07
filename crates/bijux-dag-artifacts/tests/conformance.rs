use bijux_dag_artifacts::index::{dedup_metrics_for_hashes, normalize_metadata_pairs, ArtifactPackManifest};
use bijux_dag_artifacts::lineage::{write_lineage_snapshot, ArtifactLineageEdge, ArtifactLineageSnapshot};
use bijux_dag_artifacts::paths::is_normalized_relative_path;
use bijux_dag_artifacts::proof::{ArtifactIntegrityProof, CorruptionDetectionResult, CorruptionRepairPolicy};
use bijux_dag_artifacts::schema::{validate_output_schema_descriptor, ArtifactSchemaDescriptor, SchemaValidationMode};
use bijux_dag_artifacts::{write_outputs_index, Manifest, NodeTrace, OutputsIndex, RunOutputsIndex};
use std::fs;

#[test]
fn outputs_index_is_stable_under_repeated_writes() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("b.txt"), b"b").unwrap();
    fs::write(dir.path().join("a.txt"), b"a").unwrap();

    let paths = vec!["b.txt".to_string(), "a.txt".to_string()];
    write_outputs_index(dir.path(), "node", "fp", &paths).unwrap();
    let first = fs::read_to_string(dir.path().join("index.json")).unwrap();

    write_outputs_index(dir.path(), "node", "fp", &paths).unwrap();
    let second = fs::read_to_string(dir.path().join("index.json")).unwrap();

    assert_eq!(first, second);
    let parsed: OutputsIndex = serde_json::from_str(&second).unwrap();
    assert_eq!(parsed.files.len(), 2);
    assert_eq!(parsed.files[0].path, "a.txt");
    assert_eq!(parsed.files[1].path, "b.txt");
}

#[test]
fn schema_descriptor_validation_rejects_empty_fields() {
    let descriptor = ArtifactSchemaDescriptor {
        name: "".to_string(),
        version: "v0.1".to_string(),
        media_type: "application/json".to_string(),
        encoding: "identity".to_string(),
        validation_mode: SchemaValidationMode::Strict,
    };
    assert!(validate_output_schema_descriptor(&descriptor).is_err());
}

#[test]
fn lineage_snapshot_write_is_deterministic() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("lineage.snapshot.json");
    let snapshot = ArtifactLineageSnapshot {
        schema_version: "v0.1".to_string(),
        edges: vec![ArtifactLineageEdge {
            artifact_id: "n1:out".to_string(),
            producer_node_id: "n1".to_string(),
            upstream_artifact_ids: vec!["n0:*".to_string()],
        }],
    };
    write_lineage_snapshot(&path, &snapshot).unwrap();
    let first = fs::read_to_string(&path).unwrap();
    write_lineage_snapshot(&path, &snapshot).unwrap();
    let second = fs::read_to_string(&path).unwrap();
    assert_eq!(first, second);
}

#[test]
fn metadata_normalization_and_dedup_metrics_are_stable() {
    let pairs = vec![("b".to_string(), "2".to_string()), ("a".to_string(), "1".to_string())];
    let normalized = normalize_metadata_pairs(pairs);
    assert_eq!(normalized[0].0, "a");
    assert_eq!(normalized[1].0, "b");

    let metrics = dedup_metrics_for_hashes(&[
        "h1".to_string(),
        "h1".to_string(),
        "h2".to_string(),
    ]);
    assert_eq!(metrics.total_artifacts, 3);
    assert_eq!(metrics.unique_content_hashes, 2);
    assert_eq!(metrics.deduplicated_artifacts, 1);
}

#[test]
fn proof_and_pack_contract_types_serialize() {
    let proof = ArtifactIntegrityProof {
        artifact_id: "a1".to_string(),
        file_sha256: "abc".to_string(),
        schema_name: "bijux.output.file".to_string(),
        schema_version: "v0.1".to_string(),
        producer_node_id: "node".to_string(),
        run_id: "run-1".to_string(),
    };
    let pack = ArtifactPackManifest {
        pack_manifest_version: "v0.1".to_string(),
        artifacts: vec![bijux_dag_artifacts::index::ArtifactId("a1".to_string())],
    };
    let detection = CorruptionDetectionResult {
        corrupt_detected: false,
        reason: "verified".to_string(),
    };
    let repair = CorruptionRepairPolicy {
        attempt_rebuild_from_cache: true,
        attempt_replay: true,
        fail_if_unrecoverable: true,
    };

    let payload = serde_json::json!({
        "proof": proof,
        "pack": pack,
        "detection": detection,
        "repair": repair,
    });
    assert!(payload.is_object());
}

#[test]
fn output_paths_must_be_relative_and_normalized() {
    assert!(is_normalized_relative_path("nodes/a/outputs/out.txt"));
    assert!(!is_normalized_relative_path("../escape.txt"));
    assert!(!is_normalized_relative_path("/absolute/path.txt"));
    assert!(!is_normalized_relative_path("nodes\\windows\\style.txt"));
    assert!(!is_normalized_relative_path("nodes//double//slash.txt"));
}

#[test]
fn write_outputs_index_rejects_escaping_paths() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("ok.txt"), b"ok").unwrap();
    let err = write_outputs_index(
        dir.path(),
        "node",
        "fp",
        &["ok.txt".to_string(), "../escape.txt".to_string()],
    )
    .err()
    .unwrap();
    let msg = err.to_string();
    assert!(msg.contains("path violation"));
}

#[test]
fn corruption_payloads_fail_schema_parse() {
    let truncated_manifest = r#"{\"run_id\":\"r1\",\"status\":\"ok\""#;
    let missing_trace_fields = r#"{\"node_id\":\"n1\",\"status\":\"ok\"}"#;
    let altered_outputs_index = r#"{\"files\":[{\"path\":\"out.txt\"}]}"#;

    assert!(serde_json::from_str::<Manifest>(truncated_manifest).is_err());
    assert!(serde_json::from_str::<NodeTrace>(missing_trace_fields).is_err());
    assert!(serde_json::from_str::<RunOutputsIndex>(altered_outputs_index).is_err());
}
