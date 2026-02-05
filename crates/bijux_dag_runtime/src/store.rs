use crate::io::Fs;
use bijux_dag_artifacts::RunDir;
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

    pub fn fs(&self) -> &dyn Fs {
        self.fs.as_ref()
    }

    pub fn entry(&self, key: &str) -> PathBuf {
        self.dir.join(key)
    }
}
