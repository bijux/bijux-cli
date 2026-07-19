use crate::ArtifactError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMode {
    Standard,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunFinalizationMode {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDirAuditReport {
    pub run_dir: PathBuf,
    pub valid: bool,
    pub anomalies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCleanupPlan {
    pub retained: Vec<String>,
    pub prunable: Vec<String>,
}

pub fn write_json_atomic_durable(
    path: impl AsRef<Path>,
    value: &Value,
) -> Result<(), ArtifactError> {
    let path = path.as_ref();
    let tmp = path.with_extension("tmp");
    let data = serde_json::to_vec_pretty(value)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&tmp)?;
    file.write_all(&data)?;
    file.sync_all()?;
    fs::rename(&tmp, path)?;
    Ok(())
}

pub fn write_incomplete_run_marker(
    run_dir: impl AsRef<Path>,
    reason: &str,
) -> Result<(), ArtifactError> {
    let marker_path = run_dir.as_ref().join(".run-incomplete.json");
    let payload = serde_json::json!({
        "status": "incomplete",
        "reason": reason,
    });
    write_json_atomic_durable(marker_path, &payload)
}

pub fn finalize_run_manifest(run_dir: impl AsRef<Path>) -> Result<(), ArtifactError> {
    finalize_run_manifest_with_mode(run_dir, RunFinalizationMode::Complete)
}

pub fn finalize_run_manifest_with_mode(
    run_dir: impl AsRef<Path>,
    mode: RunFinalizationMode,
) -> Result<(), ArtifactError> {
    let run_dir = run_dir.as_ref();
    let manifest = run_dir.join("manifest.json");
    if !manifest.exists() {
        return Err(ArtifactError::PathViolation(
            "manifest missing during finalization".to_string(),
        ));
    }
    let finalized = run_dir.join("manifest.finalized.json");
    fs::copy(&manifest, &finalized)?;
    let incomplete_marker_path = run_dir.join(".run-incomplete.json");
    let complete_marker_path = run_dir.join(".run-complete.json");
    match mode {
        RunFinalizationMode::Complete => {
            if incomplete_marker_path.exists() {
                fs::remove_file(&incomplete_marker_path)?;
            }
            let marker =
                serde_json::json!({"status": "complete", "manifest": "manifest.finalized.json"});
            write_json_atomic_durable(complete_marker_path, &marker)
        }
        RunFinalizationMode::Incomplete => {
            if complete_marker_path.exists() {
                fs::remove_file(&complete_marker_path)?;
            }
            if !incomplete_marker_path.exists() {
                write_incomplete_run_marker(run_dir, "run finalized with incomplete outputs")?;
            }
            Ok(())
        }
    }
}

pub fn verify_run_dir(
    run_dir: impl AsRef<Path>,
    mode: VerificationMode,
) -> Result<RunDirAuditReport, ArtifactError> {
    let run_dir = run_dir.as_ref();
    let mut anomalies = Vec::new();

    let manifest_path = run_dir.join("manifest.json");
    if !manifest_path.exists() {
        anomalies.push("missing manifest.json".to_string());
    } else {
        let manifest_text = fs::read_to_string(&manifest_path)?;
        let manifest: Value = serde_json::from_str(&manifest_text)?;
        if manifest.get("run_id").is_none() {
            anomalies.push("manifest missing run_id".to_string());
        }
        if mode == VerificationMode::Strict && manifest.get("manifest_version").is_none() {
            anomalies.push("strict mode: manifest missing manifest_version".to_string());
        }
    }

    let outputs_path = run_dir.join("outputs.index.json");
    if !outputs_path.exists() {
        anomalies.push("missing outputs.index.json".to_string());
    }

    let trace_dir = run_dir.join("trace");
    if !trace_dir.exists() {
        anomalies.push("missing trace/".to_string());
    }

    if mode == VerificationMode::Strict {
        let finalized_manifest = run_dir.join("manifest.finalized.json");
        if !finalized_manifest.exists() {
            anomalies.push("strict mode: missing manifest.finalized.json".to_string());
        }
    }

    Ok(RunDirAuditReport { run_dir: run_dir.to_path_buf(), valid: anomalies.is_empty(), anomalies })
}

pub fn build_cleanup_plan(entries: &[String], retain_prefixes: &[&str]) -> ArtifactCleanupPlan {
    let mut retained = Vec::new();
    let mut prunable = Vec::new();
    for entry in entries {
        if retain_prefixes.iter().any(|prefix| entry.starts_with(prefix)) {
            retained.push(entry.clone());
        } else {
            prunable.push(entry.clone());
        }
    }
    ArtifactCleanupPlan { retained, prunable }
}
