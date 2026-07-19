use bijux_dag_artifacts::services::{RunArtifactStore, RunArtifactVerifier};
use bijux_dag_artifacts::{ArtifactError, Manifest, RunDir};
use hex as _;
use serde as _;
use sha2 as _;
use thiserror as _;

#[derive(Default)]
struct NoopStore {
    writes: std::sync::Mutex<Vec<String>>,
}

impl RunArtifactStore for NoopStore {
    fn write_manifest(&self, run_dir: &RunDir, _manifest: &Manifest) -> Result<(), ArtifactError> {
        self.writes.lock().expect("lock").push(run_dir.final_path().display().to_string());
        Ok(())
    }
}

struct NoopVerifier;

impl RunArtifactVerifier for NoopVerifier {
    fn verify_run_dir(&self, run_dir: &std::path::Path) -> Result<(), ArtifactError> {
        if run_dir.join("manifest.json").exists() {
            Ok(())
        } else {
            Err(ArtifactError::PathViolation("manifest missing during verification".to_string()))
        }
    }
}

fn sample_manifest() -> Manifest {
    serde_json::from_str(
        r#"{
          "manifest_version":"run-manifest/v0.1",
          "run_id":"run-1",
          "created_unix_ms":1,
          "started_unix_ms":1,
          "finished_unix_ms":2,
          "graph_snapshot":"graph.snapshot.json",
          "status":"success",
          "spec":"bijux-dag/v0.1",
          "graph_fingerprint":"g",
          "tool_version":"0.1.0",
          "jobs":1,
          "adapters":[],
          "outputs":[],
          "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
          "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true,"container_image_reference_policy":"require_digest"}
        }"#,
    )
    .expect("manifest")
}

#[test]
fn run_artifact_store_trait_is_usable_as_authority_boundary() {
    let store = NoopStore::default();
    let out = tempfile::tempdir().expect("tmp");
    let run_dir = RunDir::create_with_id(out.path(), "run-1").expect("run dir");
    store.write_manifest(&run_dir, &sample_manifest()).expect("write should succeed");

    let writes = store.writes.lock().expect("lock");
    assert_eq!(writes.len(), 1);
    assert!(writes[0].contains("run-1"));
}

#[test]
fn run_artifact_verifier_trait_reports_missing_manifest() {
    let verifier = NoopVerifier;
    let dir = tempfile::tempdir().expect("tmp");

    let err = verifier.verify_run_dir(dir.path()).expect_err("missing manifest should fail");
    assert!(err.to_string().contains("manifest missing"));

    std::fs::write(dir.path().join("manifest.json"), "{}").expect("write manifest marker");
    verifier.verify_run_dir(dir.path()).expect("manifest presence should pass");
}
