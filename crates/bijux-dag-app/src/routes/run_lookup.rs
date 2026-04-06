use crate::{read_file, read_run_id, ExitCode};
use serde_json::Value;
use std::path::Path;

pub(crate) fn read_manifest_json(run_dir: &Path) -> Result<Value, ExitCode> {
    let raw = read_file(&run_dir.join("manifest.json"))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

pub(crate) fn read_run_identifier(run_dir: &Path) -> Result<String, ExitCode> {
    read_run_id(run_dir)
}

#[cfg(test)]
mod tests {
    use super::{read_manifest_json, read_run_identifier};

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
}
