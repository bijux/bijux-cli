//! Artifact identity, persistence, integrity, and lifecycle helpers for Bijux DAG.
//!
//! Prefer the crate root for focused imports, [`stable`] for the explicit
//! artifact lane, and [`prelude`] for the common read/write workflow. The
//! `experimental-public-api` feature exposes opt-in contract helpers that are
//! excluded from the default docs surface.
//!
#![allow(
    clippy::if_not_else,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::pedantic
)]
#[cfg(test)]
use bijux_dag_testkit as _;

#[doc(hidden)]
#[path = "io/fs.rs"]
pub mod fs;
#[doc(hidden)]
#[path = "storage/hardening.rs"]
pub mod hardening;
#[doc(hidden)]
#[path = "integrity/hash.rs"]
pub mod hash;
#[doc(hidden)]
#[path = "integrity/index.rs"]
pub mod index;
#[cfg(feature = "experimental-public-api")]
#[path = "integrity/iteration07_contracts.rs"]
mod iteration07_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "integrity/iteration16_contracts.rs"]
mod iteration16_contracts;
#[doc(hidden)]
#[path = "lifecycle/lineage.rs"]
pub mod lineage;
#[doc(hidden)]
#[path = "storage/models.rs"]
pub mod models;
#[doc(hidden)]
#[path = "layout/paths.rs"]
pub mod paths;
#[doc(hidden)]
#[path = "layout/platform.rs"]
pub mod platform;
#[doc(hidden)]
#[path = "lifecycle/promotion.rs"]
pub mod promotion;
#[doc(hidden)]
#[path = "integrity/proof.rs"]
pub mod proof;
#[doc(hidden)]
#[path = "lifecycle/retention.rs"]
pub mod retention;
#[doc(hidden)]
#[path = "integrity/schema.rs"]
pub mod schema;
#[doc(hidden)]
#[path = "storage/services.rs"]
pub mod services;
#[doc(hidden)]
#[path = "io/store.rs"]
pub mod store;

pub use hardening::{
    build_cleanup_plan, finalize_run_manifest, verify_run_dir, write_incomplete_run_marker,
    write_json_atomic_durable, ArtifactCleanupPlan, RunDirAuditReport, VerificationMode,
};
pub use hash::sha256_hex;
pub use index::{
    dedup_metrics_for_hashes, normalize_metadata_pairs, ArtifactId, ArtifactPackManifest,
};
pub use lineage::{write_lineage_snapshot, ArtifactLineageEdge, ArtifactLineageSnapshot};
pub use models::*;
pub use paths::is_normalized_relative_path;
pub use platform::{
    compact_lineage, explain_lineage_safe_gc, lineage_dependencies, lineage_dependents,
};
pub use proof::{ArtifactIntegrityProof, CorruptionDetectionResult, CorruptionRepairPolicy};
pub use retention::RetentionPolicy;
pub use schema::{
    validate_output_schema_descriptor, ArtifactSchemaDescriptor, SchemaValidationMode,
};
pub use services::{RunArtifactStore, RunArtifactVerifier};

use serde::Serialize;
use std::fs as std_fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Explicit long-lived artifact storage, integrity, and lifecycle surface.
pub mod stable {
    pub use crate::{
        compact_lineage, dedup_metrics_for_hashes, explain_lineage_safe_gc, lineage_dependencies,
        lineage_dependents, normalize_metadata_pairs, sha256_hex,
        validate_output_schema_descriptor, verify_run_dir, write_lineage_snapshot,
        write_outputs_index, ArtifactError, ArtifactId, ArtifactIntegrityProof,
        ArtifactLineageEdge, ArtifactLineageSnapshot, ArtifactPackManifest,
        ArtifactSchemaDescriptor, CorruptionDetectionResult, CorruptionRepairPolicy,
        RetentionPolicy, RunArtifactStore, RunArtifactVerifier, RunDir, SchemaValidationMode,
    };
}

/// Common imports for reading, writing, and validating run artifacts.
pub mod prelude {
    pub use crate::stable::{
        sha256_hex, validate_output_schema_descriptor, verify_run_dir, write_outputs_index,
        ArtifactError, ArtifactSchemaDescriptor, RunDir, SchemaValidationMode,
    };
}

/// Opt-in artifact contracts and advisory helpers that are outside the stable lane.
#[cfg(feature = "experimental-public-api")]
pub mod experimental {
    pub mod run_layout {
        pub use crate::iteration07_contracts::*;
    }
    pub mod lifecycle_and_cache {
        pub use crate::iteration16_contracts::*;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("path violation: {0}")]
    PathViolation(String),
    #[error("missing output: {0}")]
    MissingOutput(String),
}

#[derive(Debug, Clone)]
pub struct RunDir {
    staging_path: PathBuf,
    final_path: PathBuf,
}

impl RunDir {
    pub fn create(out_base: impl AsRef<Path>) -> Result<Self, ArtifactError> {
        let run_id = generate_run_id();
        Self::create_with_id(out_base, &run_id)
    }

    pub fn create_with_id(out_base: impl AsRef<Path>, run_id: &str) -> Result<Self, ArtifactError> {
        let run_id = normalize_run_id(run_id)?;
        let staging = out_base.as_ref().join(format!("run.tmp-{}", run_id));
        let final_path = out_base.as_ref().join(format!("run-{}", run_id));
        std_fs::create_dir_all(staging.join("nodes"))?;
        Ok(Self { staging_path: staging, final_path })
    }

    pub fn staging_path(&self) -> &Path {
        &self.staging_path
    }

    pub fn final_path(&self) -> &Path {
        &self.final_path
    }

    pub fn write_manifest(&self, manifest: &Manifest) -> Result<(), ArtifactError> {
        let path = self.staging_path.join("manifest.json");
        write_json_atomic(path, manifest)
    }

    pub fn write_graph_snapshot(&self, graph_json: &str) -> Result<(), ArtifactError> {
        let path = self.staging_path.join("graph.snapshot.json");
        write_bytes_atomic(path, graph_json.as_bytes())
    }

    pub fn node_dir(&self, node_id: &str) -> PathBuf {
        self.staging_path.join("nodes").join(node_id)
    }

    pub fn node_outputs_dir(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("outputs")
    }

    pub fn node_inputs_dir(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("inputs")
    }

    pub fn node_work_dir(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("work")
    }

    pub fn node_stdout_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("stdout.log")
    }

    pub fn node_stderr_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("stderr.log")
    }

    pub fn node_trace_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("trace.json")
    }

    pub fn node_resolved_params_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("resolved_params.json")
    }

    pub fn node_attempts_path(&self, node_id: &str) -> PathBuf {
        self.node_dir(node_id).join("attempts.json")
    }

    pub fn run_log_path(&self) -> PathBuf {
        self.staging_path.join("run.log.jsonl")
    }

    pub fn run_outputs_index_path(&self) -> PathBuf {
        self.staging_path.join("outputs").join("index.json")
    }

    pub fn provenance_path(&self) -> PathBuf {
        self.staging_path.join("provenance.json")
    }

    pub fn node_outputs_index_path(&self, node_id: &str) -> PathBuf {
        self.node_outputs_dir(node_id).join("index.json")
    }

    pub fn node_output_relpath(&self, node_id: &str, file: &str) -> String {
        paths::node_output_relpath(node_id, file)
    }

    pub fn node_inputs_index_path(&self, node_id: &str) -> PathBuf {
        self.node_inputs_dir(node_id).join("index.json")
    }

    pub fn finalize(self) -> Result<PathBuf, ArtifactError> {
        if let Some(parent) = self.final_path.parent() {
            std_fs::create_dir_all(parent)?;
        }
        std_fs::rename(&self.staging_path, &self.final_path)?;
        Ok(self.final_path)
    }
}

fn write_json<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), ArtifactError> {
    let data = serde_json::to_vec_pretty(value)?;
    let mut f = std_fs::File::create(path)?;
    f.write_all(&data)?;
    Ok(())
}

fn write_json_atomic<T: Serialize>(path: impl AsRef<Path>, value: &T) -> Result<(), ArtifactError> {
    let path = path.as_ref();
    let tmp = path.with_extension("tmp");
    write_json(&tmp, value)?;
    std_fs::rename(tmp, path)?;
    Ok(())
}

fn write_bytes_atomic(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), ArtifactError> {
    let path = path.as_ref();
    let tmp = path.with_extension("tmp");
    let mut file = std_fs::File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std_fs::rename(tmp, path)?;
    Ok(())
}

pub fn write_outputs_index(
    dir: impl AsRef<Path>,
    node_id: &str,
    node_fingerprint: &str,
    output_paths: &[String],
) -> Result<(), ArtifactError> {
    let mut files = Vec::new();
    for rel in output_paths {
        if !paths::is_normalized_relative_path(rel) {
            return Err(ArtifactError::PathViolation(format!(
                "output path must be normalized relative path: {rel}"
            )));
        }
        let path = dir.as_ref().join(rel);
        if !path.is_file() {
            return Err(ArtifactError::MissingOutput(rel.clone()));
        }
        let data = std_fs::read(&path)?;
        let sha = sha256_bytes(&data);
        files.push(OutputFile {
            path: rel.clone(),
            sha256: sha,
            node_id: node_id.to_string(),
            node_fingerprint: node_fingerprint.to_string(),
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let index = OutputsIndex { files };
    write_json(dir.as_ref().join("index.json"), &index)
}

pub fn write_run_outputs_index(
    dir: impl AsRef<Path>,
    index: &RunOutputsIndex,
) -> Result<(), ArtifactError> {
    let dir = dir.as_ref();
    std_fs::create_dir_all(dir)?;
    write_json_atomic(dir.join("index.json"), index)
}

pub fn write_provenance(path: impl AsRef<Path>, prov: &Provenance) -> Result<(), ArtifactError> {
    write_json_atomic(path, prov)
}

pub fn write_run_schema_index(
    path: impl AsRef<Path>,
    schema: &RunDirSchemaIndex,
) -> Result<(), ArtifactError> {
    write_json_atomic(path, schema)
}

pub fn write_inputs_index(dir: impl AsRef<Path>, index: &InputsIndex) -> Result<(), ArtifactError> {
    write_json_atomic(dir.as_ref().join("index.json"), index)
}

pub fn now_unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

pub fn build_artifact_identity(
    run_id: &str,
    node_id: &str,
    output_path: &str,
    node_fingerprint: &str,
    artifact_sha256: &str,
) -> ArtifactIdentity {
    let output_name = Path::new(output_path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(output_path)
        .to_string();
    let legacy_artifact_id = format!("{node_id}:{output_name}");
    let canonical_artifact_id =
        format!("run={run_id};node={node_id};path={output_path};sha256={artifact_sha256}");
    ArtifactIdentity {
        canonical_artifact_id,
        legacy_artifact_id,
        run_id: run_id.to_string(),
        node_id: node_id.to_string(),
        output_name,
        output_path: output_path.to_string(),
        node_fingerprint: node_fingerprint.to_string(),
        artifact_sha256: artifact_sha256.to_string(),
    }
}

fn generate_run_id() -> String {
    static RUN_ID_COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = RUN_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}-{:06}", now_unix_ms(), std::process::id(), seq % 1_000_000)
}

fn normalize_run_id(run_id: &str) -> Result<String, ArtifactError> {
    let trimmed = run_id.trim();
    if trimmed.is_empty() {
        return Err(ArtifactError::PathViolation("run id must not be empty".to_string()));
    }
    let normalized = trimmed.strip_prefix("run-").unwrap_or(trimmed);
    if normalized.is_empty()
        || normalized.contains('/')
        || normalized.contains('\\')
        || normalized.contains("..")
        || !normalized.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(ArtifactError::PathViolation(format!("invalid run id: {run_id}")));
    }
    Ok(normalized.to_string())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    hash::sha256_hex(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_finalize() {
        let dir = tempfile::tempdir().unwrap();
        let run = RunDir::create(dir.path()).unwrap();
        assert!(run.staging_path().exists());
        let final_path = run.finalize().unwrap();
        assert!(final_path.exists());
    }

    #[test]
    fn generated_run_ids_do_not_collide_within_process() {
        let first = generate_run_id();
        let second = generate_run_id();
        assert_ne!(first, second);
        assert!(first.contains('-'));
        assert!(second.contains('-'));
    }

    #[test]
    fn graph_snapshot_writes_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let run = RunDir::create(dir.path()).unwrap();
        run.write_graph_snapshot("{\"graph\":\"first\"}").unwrap();
        run.write_graph_snapshot("{\"graph\":\"second\"}").unwrap();
        let snapshot =
            std_fs::read_to_string(run.staging_path().join("graph.snapshot.json")).unwrap();
        assert_eq!(snapshot, "{\"graph\":\"second\"}");
        assert!(!run.staging_path().join("graph.snapshot.tmp").exists());
    }

    #[test]
    fn explicit_run_ids_are_normalized_and_validated() {
        let dir = tempfile::tempdir().unwrap();
        let run = RunDir::create_with_id(dir.path(), "run-2026_04").unwrap();
        assert!(run.final_path().ends_with("run-2026_04"));

        let err = RunDir::create_with_id(dir.path(), "../escape").unwrap_err();
        assert!(err.to_string().contains("invalid run id"));
    }

    #[test]
    fn provenance_and_indexes_replace_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let run = RunDir::create(dir.path()).unwrap();
        let outputs_dir = run.staging_path().join("outputs");
        let inputs_dir = run.node_inputs_dir("node");
        let provenance = Provenance {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            rustc: "rustc".to_string(),
            tool_version: "0.1.0".to_string(),
            planner_contract_version: Some("bijux-dag-planner/v1".to_string()),
            graph_fingerprint: None,
            planner_fingerprint: None,
            execution_fingerprint: None,
            evidence_fingerprint: None,
            runtime_fingerprint: None,
            policy_fingerprint: None,
            adapters: Vec::new(),
            policy: PolicyInfo {
                deny_network: true,
                deny_env: true,
                deny_clock: true,
                clean_env: true,
            },
            time_source: "system_clock".to_string(),
        };
        let run_outputs = RunOutputsIndex { files: Vec::new() };
        let inputs = InputsIndex { files: Vec::new() };

        write_provenance(run.provenance_path(), &provenance).unwrap();
        write_run_outputs_index(&outputs_dir, &run_outputs).unwrap();
        std_fs::create_dir_all(&inputs_dir).unwrap();
        write_inputs_index(&inputs_dir, &inputs).unwrap();

        assert!(!run.staging_path().join("provenance.tmp").exists());
        assert!(!outputs_dir.join("index.tmp").exists());
        assert!(!inputs_dir.join("index.tmp").exists());
    }
}
