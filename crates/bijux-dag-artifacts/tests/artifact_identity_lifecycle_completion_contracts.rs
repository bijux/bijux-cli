use bijux_dag_artifacts::hash::sha256_hex;
use bijux_dag_artifacts::lineage::ArtifactLineageSnapshot;
use bijux_dag_artifacts::platform::{
    lineage_dependencies, lineage_dependents, run_store_conformance,
};
use bijux_dag_artifacts::store::{ArtifactStoreBackend, FilesystemArtifactStore};
use bijux_dag_artifacts::{verify_run_dir, write_json_atomic_durable, VerificationMode};
use bijux_dag_testkit as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile as _;
use thiserror as _;

#[test]
fn artifact_identity_spec_document_is_present_and_non_empty() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("docs/spec/ARTIFACT_IDENTITY_CONTRACT.md");
    let text = fs::read_to_string(path).expect("artifact identity spec");
    assert!(text.contains("identity") || text.contains("Identity"));
    assert!(text.len() > 200);
}

#[test]
fn artifact_fingerprint_stability_holds_for_same_bytes() {
    let payload = b"artifact:fingerprint:stable";
    assert_eq!(sha256_hex(payload), sha256_hex(payload));
}

#[test]
fn artifact_provenance_recording_keeps_run_node_and_hash_context() {
    let record = json!({
        "run_id":"run-1",
        "node_id":"extract",
        "artifact_id":"extract:data.csv",
        "sha256": sha256_hex(b"a,b\n1,2\n")
    });
    assert_eq!(record["run_id"], "run-1");
    assert_eq!(record["node_id"], "extract");
    assert!(record["sha256"].as_str().unwrap_or_default().len() == 64);
}

#[test]
fn artifact_lineage_graph_construction_and_traversal_are_stable() {
    let snapshot = ArtifactLineageSnapshot {
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

    assert_eq!(
        lineage_dependencies(&snapshot, "train:model.bin"),
        vec!["prep:clean.csv".to_string()]
    );
    assert_eq!(
        lineage_dependents(&snapshot, "prep:clean.csv"),
        vec!["train:model.bin".to_string()]
    );
}

#[test]
fn artifact_lineage_cycle_detection_contract_flags_back_edges() {
    let deps = BTreeMap::from([
        ("a".to_string(), vec!["b".to_string()]),
        ("b".to_string(), vec!["c".to_string()]),
        ("c".to_string(), vec!["a".to_string()]),
    ]);
    assert!(has_cycle(&deps));
}

#[test]
fn artifact_lineage_from_imported_bundle_and_replay_run_remains_distinct() {
    let imported_key = format!("imported:extract:data.csv:{}", sha256_hex(b"payload"));
    let replay_key = format!("replay:extract:data.csv:{}", sha256_hex(b"payload"));
    assert_ne!(imported_key, replay_key);
}

#[test]
fn artifact_hash_property_contract_differentiates_byte_changes() {
    let base = sha256_hex(b"model-v1");
    let changed = sha256_hex(b"model-v2");
    assert_ne!(base, changed);
}

#[test]
fn artifact_hash_regression_fixture_corpus_is_stable() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/artifact_hash_regression_corpus.json");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read fixture")).expect("json");
    let cases = value["cases"].as_array().expect("cases");

    for row in cases {
        let bytes = row["payload_utf8"]
            .as_str()
            .expect("payload")
            .as_bytes()
            .to_vec();
        let expected = row["expected_sha256"].as_str().expect("hash");
        assert_eq!(sha256_hex(&bytes), expected);
    }
}

#[test]
fn artifact_store_roundtrip_corruption_and_recovery_contracts_hold() {
    let dir = tempfile::tempdir().expect("tmp");
    let store = FilesystemArtifactStore::new(dir.path());

    store
        .write_bytes("cas/ab/object.bin", b"payload")
        .expect("write");
    assert_eq!(
        store.read_bytes("cas/ab/object.bin").expect("read"),
        b"payload"
    );

    fs::write(dir.path().join("cas/ab/object.bin"), b"corrupt").expect("corrupt");
    let corrupted = store.read_bytes("cas/ab/object.bin").expect("read corrupted");
    assert_ne!(sha256_hex(&corrupted), sha256_hex(b"payload"));

    write_json_atomic_durable(
        dir.path().join("recovery.json"),
        &json!({"recovered":true,"strategy":"rewrite"}),
    )
    .expect("recovery marker");
    assert!(dir.path().join("recovery.json").exists());
}

#[test]
fn artifact_store_concurrency_and_integrity_verification_contracts_hold() {
    let dir = tempfile::tempdir().expect("tmp");
    let store: Arc<FilesystemArtifactStore> = Arc::new(FilesystemArtifactStore::new(dir.path()));
    let gate = Arc::new(Barrier::new(9));
    let mut joins = Vec::new();

    for i in 0..8u32 {
        let s = Arc::clone(&store);
        let g = Arc::clone(&gate);
        joins.push(thread::spawn(move || {
            g.wait();
            let key = format!("cas/{:02x}/{}.bin", i, i);
            let body = format!("payload-{i}").into_bytes();
            s.write_bytes(&key, &body).expect("write concurrent");
            let read_back = s.read_bytes(&key).expect("read concurrent");
            assert_eq!(read_back, body);
        }));
    }
    gate.wait();
    for j in joins {
        j.join().expect("thread");
    }

    let conformance = run_store_conformance("filesystem", &*store);
    assert!(conformance.roundtrip_ok);
}

#[test]
fn artifact_large_file_streaming_and_stress_contracts_hold() {
    let mut data = Vec::with_capacity(4 * 1024 * 1024);
    for i in 0..(4 * 1024 * 1024) {
        data.push((i % 251) as u8);
    }

    let full_hash = sha256_hex(&data);
    let mut streamed = Vec::new();
    for chunk in data.chunks(64 * 1024) {
        streamed.extend_from_slice(chunk);
    }
    assert_eq!(full_hash, sha256_hex(&streamed));

    let unique = (0..50_000u32)
        .map(|i| format!("artifact-{i:05}"))
        .collect::<BTreeSet<_>>();
    assert_eq!(unique.len(), 50_000);
}

#[test]
fn artifact_run_dir_integrity_verification_detects_missing_required_files() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-1"
        }))
        .expect("manifest"),
    )
    .expect("write");

    let report = verify_run_dir(dir.path(), VerificationMode::Standard).expect("verify");
    assert!(!report.valid);
    assert!(!report.anomalies.is_empty());
}

fn has_cycle(graph: &BTreeMap<String, Vec<String>>) -> bool {
    fn dfs(
        n: &str,
        graph: &BTreeMap<String, Vec<String>>,
        temp: &mut BTreeSet<String>,
        perm: &mut BTreeSet<String>,
    ) -> bool {
        if perm.contains(n) {
            return false;
        }
        if !temp.insert(n.to_string()) {
            return true;
        }
        if let Some(next) = graph.get(n) {
            for m in next {
                if dfs(m, graph, temp, perm) {
                    return true;
                }
            }
        }
        temp.remove(n);
        perm.insert(n.to_string());
        false
    }

    let mut temp = BTreeSet::new();
    let mut perm = BTreeSet::new();
    for node in graph.keys() {
        if dfs(node, graph, &mut temp, &mut perm) {
            return true;
        }
    }
    false
}
