use crate::io::Fs;
use crate::io::StdFs;
use bijux_dag_artifacts::RunDir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct ArtifactStore {
    run_dir: Arc<RunDir>,
    fs: Arc<dyn Fs>,
}

impl ArtifactStore {
    pub fn new(run_dir: Arc<RunDir>, fs: Arc<dyn Fs>) -> Self {
        Self { run_dir, fs }
    }

    pub fn with_std_fs(run_dir: Arc<RunDir>) -> Self {
        Self {
            run_dir,
            fs: Arc::new(StdFs),
        }
    }

    pub fn run_dir(&self) -> &RunDir {
        &self.run_dir
    }

    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
    }

    pub fn ensure_node_dir(&self, node_id: &str) -> std::io::Result<()> {
        let dir = self.run_dir.node_dir(node_id);
        self.fs.create_dir_all(&dir)
    }

    pub fn write_trace(&self, node_id: &str, data: &[u8]) -> std::io::Result<()> {
        let path = self.run_dir.node_trace_path(node_id);
        self.fs.write(&path, data)
    }

    pub fn write_resolved_params(&self, node_id: &str, data: &[u8]) -> std::io::Result<()> {
        let path = self.run_dir.node_resolved_params_path(node_id);
        self.fs.write(&path, data)
    }

    pub fn open_run_log(&self) -> std::io::Result<std::fs::File> {
        self.fs.open_append(self.run_dir.run_log_path().as_path())
    }

    pub fn write_atomic_json(&self, relative_path: &str, payload: &[u8]) -> std::io::Result<()> {
        validate_storage_relative_path(relative_path)?;
        let target = self.run_dir.staging_path().join(relative_path);
        let tmp = target.with_extension("tmp");
        self.fs.write(&tmp, payload)?;
        self.fs.rename(&tmp, &target)
    }

    pub fn read_validated_run_manifest(&self) -> std::io::Result<serde_json::Value> {
        let path = self.run_dir.staging_path().join("manifest.json");
        let payload = self.fs.read_to_string(&path)?;
        let parsed: serde_json::Value = serde_json::from_str(&payload).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid manifest json: {err}"),
            )
        })?;
        if parsed.get("run_id").is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "manifest missing run_id",
            ));
        }
        Ok(parsed)
    }

    pub fn verify_health(&self) -> std::io::Result<StorageHealthReport> {
        let mut anomalies = Vec::new();
        let manifest = self.run_dir.staging_path().join("manifest.json");
        if self.fs.metadata(&manifest).is_err() {
            anomalies.push("missing manifest.json".to_string());
        } else if self.read_validated_run_manifest().is_err() {
            anomalies.push("invalid manifest.json".to_string());
        }
        let outputs_index = self.run_dir.run_outputs_index_path();
        if self.fs.metadata(&outputs_index).is_err() {
            anomalies.push("missing outputs.index.json".to_string());
        }
        Ok(StorageHealthReport {
            run_dir: self.run_dir.staging_path().display().to_string(),
            healthy: anomalies.is_empty(),
            anomalies,
        })
    }
}

#[derive(Clone)]
pub struct CacheStore {
    dir: PathBuf,
    fs: Arc<dyn Fs>,
}

impl CacheStore {
    pub fn new(dir: PathBuf, fs: Arc<dyn Fs>) -> Self {
        Self { dir, fs }
    }

    pub fn with_std_fs(dir: PathBuf) -> Self {
        Self {
            dir,
            fs: Arc::new(StdFs),
        }
    }

    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
    }

    pub fn entry(&self, key: &str) -> PathBuf {
        self.dir.join(key)
    }

    pub fn write_cache_meta_atomic(
        &self,
        key: &str,
        meta: &serde_json::Value,
    ) -> std::io::Result<()> {
        validate_storage_relative_path(key)?;
        let entry = self.entry(key);
        self.fs.create_dir_all(&entry)?;
        let target = entry.join("meta.json");
        let tmp = entry.join("meta.json.tmp");
        let payload = serde_json::to_vec_pretty(meta).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("serialize meta: {err}"),
            )
        })?;
        self.fs.write(&tmp, &payload)?;
        self.fs.rename(&tmp, &target)
    }

    pub fn read_validated_cache_meta(&self, key: &str) -> std::io::Result<serde_json::Value> {
        validate_storage_relative_path(key)?;
        let path = self.entry(key).join("meta.json");
        let payload = self.fs.read_to_string(&path)?;
        let parsed: serde_json::Value = serde_json::from_str(&payload).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid cache meta: {err}"),
            )
        })?;
        if parsed.get("fingerprint").is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "cache meta missing fingerprint",
            ));
        }
        Ok(parsed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageHealthReport {
    pub run_dir: String,
    pub healthy: bool,
    pub anomalies: Vec<String>,
}

pub fn validate_storage_relative_path(path: &str) -> std::io::Result<()> {
    if path.is_empty() || path.starts_with('/') || path.contains("..") || path.contains('\\') {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid storage relative path: {path}"),
        ));
    }
    Ok(())
}
