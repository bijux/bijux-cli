use bijux_dag_artifacts::prelude::{
    sha256_hex, validate_output_schema_descriptor, write_inputs_index, write_outputs_index,
    ArtifactSchemaDescriptor, RunDir, SchemaValidationMode,
};
use bijux_dag_artifacts::stable::{
    lineage_dependencies, ArtifactLineageEdge, ArtifactLineageSnapshot,
};
use bijux_dag_artifacts::{DeclaredOutputArtifact, InputFile, InputsIndex};
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[test]
fn prelude_covers_artifact_write_and_validation_flow() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("out.txt"), b"payload").expect("write output");

    let run_dir = RunDir::create_with_id(dir.path(), "api-surface").expect("run dir");
    assert!(run_dir.final_path().ends_with("run-api-surface"));

    write_inputs_index(
        dir.path(),
        &InputsIndex {
            collections: Vec::new(),
            files: vec![InputFile {
                local_path: "producer/input".to_string(),
                source_sha256: sha256_hex(b"payload"),
                source_node_id: "producer".to_string(),
                source_node_fingerprint: "fp-upstream".to_string(),
                source_output_name: "out".to_string(),
                materialization_mode: "copy".to_string(),
            }],
        },
    )
    .expect("inputs index");

    write_outputs_index(
        dir.path(),
        "node",
        "fp",
        &[DeclaredOutputArtifact {
            name: "out".to_string(),
            path: "out.txt".to_string(),
            kind: "file".to_string(),
            media_type: "application/octet-stream".to_string(),
            promotable: false,
        }],
    )
    .expect("index");
    assert_eq!(sha256_hex(b"payload").len(), 64);

    validate_output_schema_descriptor(&ArtifactSchemaDescriptor {
        name: "bijux.output.file".to_string(),
        version: "v0.1".to_string(),
        media_type: "text/plain".to_string(),
        encoding: "identity".to_string(),
        validation_mode: SchemaValidationMode::Strict,
    })
    .expect("schema descriptor");
}

#[test]
fn stable_lane_keeps_lineage_helpers_discoverable() {
    let snapshot = ArtifactLineageSnapshot {
        schema_version: "v0.1".to_string(),
        edges: vec![
            ArtifactLineageEdge {
                artifact_id: "root:out".to_string(),
                producer_node_id: "root".to_string(),
                upstream_artifact_ids: vec![],
            },
            ArtifactLineageEdge {
                artifact_id: "leaf:out".to_string(),
                producer_node_id: "leaf".to_string(),
                upstream_artifact_ids: vec!["root:out".to_string()],
            },
        ],
    };

    let deps = lineage_dependencies(&snapshot, "leaf:out");
    assert_eq!(deps, vec!["root:out".to_string()]);
}
