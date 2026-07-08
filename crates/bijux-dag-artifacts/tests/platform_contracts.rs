use bijux_dag_artifacts::index::ArtifactId;
use bijux_dag_artifacts::lineage::{ArtifactLineageEdge, ArtifactLineageSnapshot};
use bijux_dag_artifacts::platform::{
    build_replay_assist, compact_lineage, lineage_dependencies, lineage_dependents,
    plan_lineage_safe_gc, run_store_conformance,
};
use bijux_dag_artifacts::store::ArtifactStoreBackend;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[derive(Default)]
struct MemoryStore(std::sync::Mutex<std::collections::BTreeMap<String, Vec<u8>>>);

impl ArtifactStoreBackend for MemoryStore {
    fn write_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    fn read_bytes(&self, key: &str) -> Result<Vec<u8>, String> {
        self.0
            .lock()
            .map_err(|_| "lock poisoned".to_string())?
            .get(key)
            .cloned()
            .ok_or_else(|| "missing key".to_string())
    }
}

fn fixture_snapshot() -> ArtifactLineageSnapshot {
    ArtifactLineageSnapshot {
        schema_version: "v0.1".to_string(),
        edges: vec![
            ArtifactLineageEdge {
                artifact_id: "a.mid".to_string(),
                producer_node_id: "node.mid".to_string(),
                upstream_artifact_ids: vec!["a.raw".to_string()],
            },
            ArtifactLineageEdge {
                artifact_id: "a.final".to_string(),
                producer_node_id: "node.final".to_string(),
                upstream_artifact_ids: vec!["a.mid".to_string()],
            },
        ],
    }
}

#[test]
fn lineage_utilities_are_stable() {
    let snapshot = fixture_snapshot();

    let compacted = compact_lineage(&snapshot);
    assert_eq!(compacted.artifact_count, 3);
    assert_eq!(compacted.edge_count, 2);

    let deps = lineage_dependencies(&snapshot, "a.final");
    assert_eq!(deps, vec!["a.mid".to_string()]);

    let dependents = lineage_dependents(&snapshot, "a.mid");
    assert_eq!(dependents, vec!["a.final".to_string()]);

    let assist = build_replay_assist(&snapshot, ArtifactId("a.final".to_string()));
    assert_eq!(assist.required_upstream_artifacts, vec![ArtifactId("a.mid".to_string())]);
    assert_eq!(assist.required_nodes, vec!["node.final".to_string()]);
}

#[test]
fn gc_planning_preserves_referenced_artifacts() {
    let all = vec![
        ArtifactId("a.raw".to_string()),
        ArtifactId("a.mid".to_string()),
        ArtifactId("a.final".to_string()),
    ];
    let referenced = vec![ArtifactId("a.final".to_string())];
    let plan = plan_lineage_safe_gc(&referenced, &all, "lineage-1");

    assert_eq!(plan.preserved_artifacts, referenced);
    assert_eq!(plan.collectable_artifacts.len(), 2);
    assert_eq!(plan.lineage_snapshot_id, "lineage-1");
}

#[test]
fn store_conformance_report_roundtrip() {
    let backend = MemoryStore::default();
    let report = run_store_conformance("memory", &backend);

    assert!(report.write_ok);
    assert!(report.read_ok);
    assert!(report.roundtrip_ok);
    assert!(report.errors.is_empty());
}
