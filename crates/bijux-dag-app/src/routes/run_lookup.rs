use crate::{read_file, read_run_id, ExitCode};
use bijux_dag_artifacts::RunDirLayout;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunWorkspacePaths {
    requested_run_id: String,
    pub normalized_run_id: String,
    pub staging_path: PathBuf,
    pub final_path: PathBuf,
}

impl RunWorkspacePaths {
    pub(crate) fn for_run(root: &Path, run_id: &str) -> Result<Self, ExitCode> {
        let layout = RunDirLayout::preview(root, Some(run_id)).map_err(|_| ExitCode::from(2))?;
        Ok(Self {
            requested_run_id: run_id.trim().to_string(),
            normalized_run_id: layout.run_id,
            staging_path: layout.staging_path,
            final_path: layout.final_path,
        })
    }

    pub(crate) fn preferred_read_path(&self) -> PathBuf {
        if self.final_path.exists() {
            return self.final_path.clone();
        }
        if self.staging_path.exists() {
            return self.staging_path.clone();
        }
        let requested_path = self.requested_path();
        if requested_path.exists() {
            return requested_path;
        }
        self.final_path.clone()
    }

    pub(crate) fn active_run_path(&self) -> Option<PathBuf> {
        self.staging_path.exists().then(|| self.staging_path.clone())
    }

    pub(crate) fn stable_run_path(&self) -> Option<PathBuf> {
        if self.final_path.exists() {
            return Some(self.final_path.clone());
        }
        let requested_path = self.requested_path();
        requested_path.exists().then_some(requested_path)
    }

    fn requested_path(&self) -> PathBuf {
        self.final_path.parent().unwrap_or_else(|| Path::new(".")).join(&self.requested_run_id)
    }
}

pub(crate) fn read_manifest_json(run_dir: &Path) -> Result<Value, ExitCode> {
    let raw = read_file(&run_dir.join("manifest.json"))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

pub(crate) fn read_run_identifier(run_dir: &Path) -> Result<String, ExitCode> {
    read_run_id(run_dir)
}

#[cfg(test)]
mod tests {
    use super::{read_manifest_json, read_run_identifier, RunWorkspacePaths};

    #[test]
    fn reads_manifest_and_run_id() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("manifest.json"), r#"{"run_id":"r-123","status":"ok"}"#)
            .expect("write manifest");

        let manifest = read_manifest_json(tmp.path()).expect("read manifest");
        assert_eq!(manifest.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(read_run_identifier(tmp.path()).expect("read run id"), "r-123");
    }

    #[test]
    fn malformed_manifest_is_rejected_without_panic() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("manifest.json"), b"{bad-json").expect("write manifest");
        assert!(read_manifest_json(tmp.path()).is_err());
    }

    #[test]
    fn missing_run_id_is_rejected() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("manifest.json"), r#"{"status":"ok"}"#)
            .expect("write manifest");
        assert!(read_run_identifier(tmp.path()).is_err());
    }

    #[test]
    fn workspace_paths_accept_prefixed_and_canonical_run_ids() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prefixed = RunWorkspacePaths::for_run(tmp.path(), "run-abc").expect("prefixed paths");
        assert_eq!(prefixed.normalized_run_id, "abc");
        assert!(prefixed.staging_path.ends_with("run.tmp-abc"));
        assert!(prefixed.final_path.ends_with("run-abc"));

        let canonical = RunWorkspacePaths::for_run(tmp.path(), "abc").expect("canonical paths");
        assert_eq!(canonical.normalized_run_id, "abc");
        assert_eq!(canonical.staging_path, prefixed.staging_path);
        assert_eq!(canonical.final_path, prefixed.final_path);
    }

    #[test]
    fn workspace_paths_prefer_final_then_staging_then_requested_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let paths = RunWorkspacePaths::for_run(tmp.path(), "run-history").expect("paths");

        assert_eq!(paths.preferred_read_path(), paths.final_path);

        std::fs::create_dir_all(&paths.staging_path).expect("create staging");
        assert_eq!(paths.preferred_read_path(), paths.staging_path);
        std::fs::remove_dir_all(&paths.staging_path).expect("remove staging");

        std::fs::create_dir_all(tmp.path().join("run-history")).expect("create requested");
        assert_eq!(paths.preferred_read_path(), tmp.path().join("run-history"));
    }
}
