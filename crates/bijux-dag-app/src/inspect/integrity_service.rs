use crate::run_data::load_snapshot;
use crate::{read_file, ExitCode};
use bijux_dag_artifacts::{Manifest, RunOutputsIndex};
use bijux_dag_core::Effect;
use bijux_dag_runtime::{invariants, NodeStatus};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub(crate) fn hash_run_dir(run_dir: &Path) -> Result<String, ExitCode> {
    let mut hasher = Sha256::new();
    for rel in ["manifest.json", "graph.snapshot.json", "outputs/index.json"] {
        let path = run_dir.join(rel);
        if path.exists() {
            let bytes = fs::read(path).map_err(|_| ExitCode::from(3))?;
            hasher.update(bytes);
        }
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn inspect_artifact(run_dir: &Path, artifact_id: &str) -> Result<Value, ExitCode> {
    let (node_id, file_name) = artifact_id
        .split_once(':')
        .ok_or_else(|| ExitCode::from(2))?;
    let manifest_raw = read_file(&run_dir.join("manifest.json"))?;
    let manifest: Manifest = serde_json::from_str(&manifest_raw).map_err(|_| ExitCode::from(3))?;
    let run_outputs_raw = read_file(&run_dir.join("outputs").join("index.json"))?;
    let run_outputs: RunOutputsIndex =
        serde_json::from_str(&run_outputs_raw).map_err(|_| ExitCode::from(3))?;
    let output = run_outputs
        .files
        .iter()
        .find(|entry| entry.node_id == node_id && entry.path.ends_with(&format!("/{file_name}")))
        .ok_or_else(|| ExitCode::from(3))?;
    let artifact_path = run_dir.join(&output.path);
    let (size_bytes, payload_missing) = match fs::metadata(&artifact_path) {
        Ok(metadata) => (Some(metadata.len()), false),
        Err(_) => (None, true),
    };
    let lineage_path = run_dir.join("lineage.snapshot.json");
    let lineage = if lineage_path.exists() {
        let data = read_file(&lineage_path)?;
        let snapshot: bijux_dag_artifacts::lineage::ArtifactLineageSnapshot =
            serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
        let upstream = bijux_dag_artifacts::platform::lineage_dependencies(&snapshot, artifact_id);
        let downstream = bijux_dag_artifacts::platform::lineage_dependents(&snapshot, artifact_id);
        json!({
            "upstream_artifact_ids": upstream,
            "downstream_artifact_ids": downstream
        })
    } else {
        json!({
            "upstream_artifact_ids": [],
            "downstream_artifact_ids": []
        })
    };
    let run_id = manifest.run_id.clone();
    Ok(json!({
        "artifact_id": artifact_id,
        "artifact_sha256": output.sha256,
        "node_id": output.node_id,
        "node_fingerprint": output.node_fingerprint,
        "path": output.path,
        "size_bytes": size_bytes,
        "payload_missing": payload_missing,
        "provenance": {
            "graph_fingerprint": manifest.graph_fingerprint,
            "run_id": run_id,
            "attempt": 0
        },
        "identity_explain": {
            "artifact_id": artifact_id,
            "composed_from": {
                "run_id": manifest.run_id,
                "node_id": output.node_id,
                "node_fingerprint": output.node_fingerprint,
                "artifact_sha256": output.sha256,
                "path": output.path
            },
            "hash_algorithm": "sha256",
            "identity_scope": "artifact content + provenance"
        },
        "lineage": lineage
    }))
}

pub(crate) fn check_engine(bin: &str) -> Value {
    match std::process::Command::new(bin).arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            json!({"status":"ok","version":v})
        }
        _ => json!({"status":"missing"}),
    }
}

pub(crate) fn verify_run(run_dir: &Path, deep: bool, strict: bool) -> Result<Value, ExitCode> {
    let mut errors = Vec::new();
    let mut invariant_violations = Vec::new();
    let manifest_path = run_dir.join("manifest.json");
    let manifest_data = fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?;
    let manifest: Manifest = serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
    if manifest.created_unix_ms > manifest.started_unix_ms
        || manifest.started_unix_ms > manifest.finished_unix_ms
    {
        errors.push("manifest timestamps are not monotonic".to_string());
    }
    if let Some(dir_name) = run_dir.file_name().and_then(|v| v.to_str()) {
        if let Some(expected) = dir_name.strip_prefix("run-") {
            if manifest.run_id != expected {
                errors.push("manifest run_id does not match finalized run directory".to_string());
            }
        }
    }
    let snapshot = load_snapshot(run_dir)?;
    let computed = snapshot.graph.graph_fingerprint().unwrap_or_default();
    if computed != snapshot.graph_fingerprint {
        errors.push(format!(
            "graph_fingerprint mismatch: {} != {}",
            computed, snapshot.graph_fingerprint
        ));
    }
    for node in &snapshot.graph.nodes {
        if manifest.policy.deny_network && node.effects.contains(&Effect::Network) {
            errors.push(format!("policy deny_network violated by node {}", node.id));
        }
        if manifest.policy.deny_env && node.effects.contains(&Effect::Env) {
            errors.push(format!("policy deny_env violated by node {}", node.id));
        }
        if manifest.policy.deny_clock && node.effects.contains(&Effect::Clock) {
            errors.push(format!("policy deny_clock violated by node {}", node.id));
        }
    }

    let outputs_index_path = run_dir.join("outputs").join("index.json");
    let mut outputs_count = 0usize;
    if !outputs_index_path.exists() {
        errors.push("missing outputs/index.json".to_string());
    } else {
        let data = fs::read_to_string(&outputs_index_path).map_err(|_| ExitCode::from(3))?;
        let index: RunOutputsIndex = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
        outputs_count = index.files.len();
        if deep {
            let mut sorted = index.files.clone();
            sorted.sort_by(|a, b| a.path.cmp(&b.path));
            if sorted != index.files {
                errors.push("outputs/index.json is not canonically ordered".to_string());
            }
        }
        for file in index.files {
            if deep && !bijux_dag_artifacts::paths::is_normalized_relative_path(&file.path) {
                errors.push(format!(
                    "output path is not normalized relative path: {}",
                    file.path
                ));
            }
            let path = run_dir.join(&file.path);
            if !path.exists() {
                errors.push(format!("missing output file: {}", file.path));
                continue;
            }
            let bytes = fs::read(&path).map_err(|_| ExitCode::from(3))?;
            let sha = sha256_bytes(&bytes);
            if sha != file.sha256 {
                errors.push(format!("hash mismatch: {}", file.path));
            }
        }
    }

    let nodes_dir = run_dir.join("nodes");
    let mut observed_statuses = Vec::new();
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let trace_path = entry.path().join("trace.json");
            if !trace_path.exists() {
                errors.push(format!(
                    "missing trace: {}",
                    entry.file_name().to_string_lossy()
                ));
                continue;
            }
            let data = fs::read_to_string(&trace_path).map_err(|_| ExitCode::from(3))?;
            let val: Value = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
            if let Some(status) = val.get("status").and_then(|s| s.as_str()) {
                match status {
                    "success" => observed_statuses.push(NodeStatus::Success),
                    "failed" => observed_statuses.push(NodeStatus::Failed),
                    "skipped" => observed_statuses.push(NodeStatus::Skipped),
                    "cached" => observed_statuses.push(NodeStatus::Cached),
                    _ => {}
                }
            }
            if deep {
                let typed_parse: Result<bijux_dag_artifacts::NodeTrace, _> =
                    serde_json::from_str(&data);
                if typed_parse.is_err() {
                    errors.push(format!(
                        "trace schema parse failed: {}",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
            for key in [
                "node_id",
                "status",
                "started_unix_ms",
                "finished_unix_ms",
                "fingerprint",
            ] {
                if val.get(key).is_none() {
                    errors.push(format!(
                        "trace missing {}: {}",
                        key,
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
            if deep {
                let started = val
                    .get("started_unix_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let finished = val
                    .get("finished_unix_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if !invariants::trace_time_order_ok(started, finished) {
                    invariant_violations.push(format!(
                        "INV-TRACE-TIME-001 violation in {}",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
        }
    }

    let manifest_counts = invariants::RunNodeCounts {
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
    };
    if !invariants::run_summary_invariant_ok(manifest_counts, &observed_statuses) {
        invariant_violations
            .push("INV-RUN-COUNTS-001 manifest totals do not match node traces".to_string());
    }
    if manifest.status == "completed"
        && !invariants::terminal_run_has_terminal_node(&observed_statuses)
    {
        invariant_violations
            .push("INV-RUN-TERMINAL-001 completed run has no terminal node statuses".to_string());
    }

    if deep || strict {
        if serde_json::from_str::<Manifest>(&manifest_data).is_err() {
            errors.push("manifest schema parse failed".to_string());
        }
        if !outputs_index_path.exists() {
            errors.push("deep verify requires outputs/index.json".to_string());
        }
    }
    if strict {
        for rel in ["graph.snapshot.json", "nodes"] {
            if !run_dir.join(rel).exists() {
                errors.push(format!(
                    "strict verify missing required run artifact: {}",
                    rel
                ));
            }
        }
        if manifest.manifest_version != "run-manifest/v0.1" {
            errors.push("strict verify unsupported manifest_version".to_string());
        }
        for rel in ["observability.timeline.json", "observability.events.json"] {
            if !run_dir.join(rel).exists() {
                errors.push(format!(
                    "strict verify missing required run artifact: {}",
                    rel
                ));
            }
        }
        if manifest.status == "failed" && !run_dir.join("observability.root-causes.json").exists() {
            errors.push(
                "strict verify missing required run artifact: observability.root-causes.json"
                    .to_string(),
            );
        }
    }
    if deep || strict {
        let manifest_json: Value =
            serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
        if let Some(summary) = manifest_json
            .get("run_metadata")
            .and_then(|m| m.get("environment_summary"))
        {
            let summary_bytes = serde_json::to_vec(summary).map_err(|_| ExitCode::from(3))?;
            let expected = sha256_bytes(&summary_bytes);
            let actual = manifest_json
                .get("run_metadata")
                .and_then(|m| m.get("environment_summary_sha256"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if actual != expected {
                errors.push("environment summary checksum mismatch".to_string());
            }
        }
    }

    let status = if errors.is_empty() && invariant_violations.is_empty() {
        "ok"
    } else {
        "error"
    };
    Ok(json!({
        "status": status,
        "mode": if strict {
            "strict"
        } else if deep {
            "deep"
        } else {
            "standard"
        },
        "artifacts_checked": {
            "manifest": manifest_path.exists(),
            "outputs_index": outputs_index_path.exists(),
            "outputs_files": outputs_count
        },
        "errors": errors,
        "invariant_violations": invariant_violations
    }))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}
