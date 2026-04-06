#![allow(
    clippy::if_not_else,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::pedantic
)]
#[cfg(test)]
use bijux_dag_testkit as _;

#[path = "io/fs.rs"]
pub mod fs;
#[path = "storage/hardening.rs"]
pub mod hardening;
#[path = "integrity/hash.rs"]
pub mod hash;
#[path = "integrity/index.rs"]
pub mod index;
#[path = "lifecycle/lineage.rs"]
pub mod lineage;
#[path = "storage/models.rs"]
pub mod models;
#[path = "layout/paths.rs"]
pub mod paths;
#[path = "layout/platform.rs"]
pub mod platform;
#[path = "lifecycle/promotion.rs"]
pub mod promotion;
#[path = "integrity/proof.rs"]
pub mod proof;
#[path = "lifecycle/retention.rs"]
pub mod retention;
#[path = "integrity/schema.rs"]
pub mod schema;
#[path = "storage/services.rs"]
pub mod services;
#[path = "io/store.rs"]
pub mod store;

pub use hardening::{
    build_cleanup_plan, finalize_run_manifest, verify_run_dir, write_incomplete_run_marker,
    write_json_atomic_durable, ArtifactCleanupPlan, RunDirAuditReport, VerificationMode,
};
pub use models::*;

use serde::Serialize;
use std::fs as std_fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("path violation: {0}")]
    PathViolation(String),
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
        let staging = out_base.as_ref().join(format!("run.tmp-{}", run_id));
        let final_path = out_base.as_ref().join(format!("run-{}", run_id));
        std_fs::create_dir_all(staging.join("nodes"))?;
        Ok(Self {
            staging_path: staging,
            final_path,
        })
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
        let mut f = std_fs::File::create(path)?;
        f.write_all(graph_json.as_bytes())?;
        Ok(())
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
            continue;
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
    write_json(dir.join("index.json"), index)
}

pub fn write_provenance(path: impl AsRef<Path>, prov: &Provenance) -> Result<(), ArtifactError> {
    write_json(path, prov)
}

pub fn write_inputs_index(dir: impl AsRef<Path>, index: &InputsIndex) -> Result<(), ArtifactError> {
    write_json(dir.as_ref().join("index.json"), index)
}

pub fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn generate_run_id() -> String {
    now_unix_ms().to_string()
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
}
