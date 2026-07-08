use bijux_dag_artifacts::fs::node_output_relpath;
use bijux_dag_artifacts::store::{
    ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore, ObjectArtifactStore,
};
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[test]
fn filesystem_store_roundtrip_and_capability_flags_are_implemented() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = FilesystemArtifactStore::new(dir.path());
    store.write_bytes("runs/run-1/nodes/a/out.bin", b"payload").expect("write");
    let data = store.read_bytes("runs/run-1/nodes/a/out.bin").expect("read");
    assert_eq!(data, b"payload");

    let caps = store.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::Implemented);
    assert!(caps.can_write_bytes);
    assert!(caps.can_read_bytes);
}

#[test]
fn object_store_surfaces_modeled_only_errors() {
    let store =
        ObjectArtifactStore { bucket: "demo".to_string(), prefix: "artifacts/".to_string() };
    let write_err =
        store.write_bytes("x", b"payload").expect_err("modeled store write should fail");
    let read_err = store.read_bytes("x").expect_err("modeled store read should fail");

    assert!(write_err.contains("not implemented"));
    assert!(read_err.contains("modeled-only"));

    let caps = store.capabilities();
    assert_eq!(caps.support_level, ArtifactStoreSupportLevel::ModeledOnly);
    assert!(!caps.can_write_bytes);
    assert!(!caps.can_read_bytes);
}

#[test]
fn io_fs_node_output_relpath_is_stable() {
    let rel = node_output_relpath("extract", "data.csv");
    assert_eq!(rel, "nodes/extract/outputs/data.csv");
}
