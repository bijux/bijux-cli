use crate::diff::{build_run_diff, RunDiff};
use bijux_dag_artifacts::OutputsIndex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Deserialize)]
struct GraphSnapshot {
    graph_fingerprint: String,
}

pub(crate) fn run_diff_from_dirs(run_a: &Path, run_b: &Path) -> Result<RunDiff, ExitCode> {
    let manifest_a = read_json(&run_a.join("manifest.json"))?;
    let manifest_b = read_json(&run_b.join("manifest.json"))?;
    let snap_a: GraphSnapshot = read_typed_json(&run_a.join("graph.snapshot.json"))?;
    let snap_b: GraphSnapshot = read_typed_json(&run_b.join("graph.snapshot.json"))?;
    let nodes_a = read_node_traces(run_a)?;
    let nodes_b = read_node_traces(run_b)?;
    let outputs_a = read_outputs_indexes(run_a)?;
    let outputs_b = read_outputs_indexes(run_b)?;
    Ok(build_run_diff(
        manifest_a,
        manifest_b,
        snap_a.graph_fingerprint,
        snap_b.graph_fingerprint,
        &nodes_a,
        &nodes_b,
        &outputs_a,
        &outputs_b,
    ))
}

pub(crate) fn replay_evidence_gaps(run_dir: &Path) -> Vec<String> {
    let mut gaps = BTreeSet::new();
    let manifest_path = run_dir.join("manifest.json");
    let graph_path = run_dir.join("graph.snapshot.json");
    if !manifest_path.exists() {
        gaps.insert("missing_manifest".to_string());
    }
    if !graph_path.exists() {
        gaps.insert("missing_graph".to_string());
    }

    let manifest: Option<Value> = read_json(&manifest_path).ok();
    if manifest
        .as_ref()
        .and_then(|value| value.get("policy"))
        .is_none()
    {
        gaps.insert("missing_policy".to_string());
    }

    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut trace_seen = false;
        if let Ok(entries) = fs::read_dir(&nodes_dir) {
            for entry in entries.filter_map(Result::ok) {
                let node_dir = entry.path();
                if !node_dir.is_dir() {
                    continue;
                }
                let trace_path = node_dir.join("trace.json");
                if !trace_path.exists() {
                    gaps.insert("missing_trace".to_string());
                    continue;
                }
                trace_seen = true;
                if let Ok(trace) = read_json(&trace_path) {
                    let adapter_id_missing = trace
                        .get("adapter_id")
                        .and_then(Value::as_str)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true);
                    let adapter_version_missing = trace
                        .get("adapter_version")
                        .and_then(Value::as_str)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true);
                    if adapter_id_missing || adapter_version_missing {
                        gaps.insert("missing_adapter_identity".to_string());
                    }
                    let terminal = trace
                        .get("status")
                        .and_then(Value::as_str)
                        .map(|status| matches!(status, "success" | "cached"))
                        .unwrap_or(false);
                    if terminal {
                        let outputs_index_path = node_dir.join("outputs").join("index.json");
                        if !outputs_index_path.exists() {
                            gaps.insert("missing_artifact_hash".to_string());
                        } else if let Ok(index) = read_typed_json::<OutputsIndex>(&outputs_index_path) {
                            if index.files.iter().any(|file| file.sha256.trim().is_empty()) {
                                gaps.insert("missing_artifact_hash".to_string());
                            }
                        } else {
                            gaps.insert("missing_artifact_hash".to_string());
                        }
                    }
                } else {
                    gaps.insert("missing_trace".to_string());
                }
            }
        }
        if !trace_seen {
            gaps.insert("missing_trace".to_string());
        }
    } else {
        gaps.insert("missing_trace".to_string());
    }

    gaps.into_iter().collect()
}

fn read_json(path: &Path) -> Result<Value, ExitCode> {
    let payload = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&payload).map_err(|_| ExitCode::from(3))
}

fn read_typed_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ExitCode> {
    let payload = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&payload).map_err(|_| ExitCode::from(3))
}

fn read_node_traces(run_dir: &Path) -> Result<HashMap<String, Value>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(map);
    }
    let mut entries: Vec<_> = fs::read_dir(nodes_dir)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        if trace_path.exists() {
            map.insert(node_id, read_json(&trace_path)?);
        }
    }
    Ok(map)
}

fn read_outputs_indexes(run_dir: &Path) -> Result<HashMap<String, OutputsIndex>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(map);
    }
    let mut entries: Vec<_> = fs::read_dir(nodes_dir)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let index_path = entry.path().join("outputs").join("index.json");
        if index_path.exists() {
            map.insert(node_id, read_typed_json(&index_path)?);
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::{replay_evidence_gaps, run_diff_from_dirs};
    use std::fs;

    fn write(path: &std::path::Path, value: &str) {
        fs::write(path, value).expect("write test file");
    }

    #[test]
    fn replay_service_marks_identical_runs_equivalent() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(&run_a).expect("create run-a");
        fs::create_dir_all(&run_b).expect("create run-b");
        write(&run_a.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
    }

    #[test]
    fn replay_service_reports_graph_mismatch() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(&run_a).expect("create run-a");
        fs::create_dir_all(&run_b).expect("create run-b");
        write(&run_a.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-2"}"#);

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff
            .replay_equivalence
            .reasons
            .iter()
            .any(|reason| { reason.contains("graph fingerprint differs") }));
    }

    #[test]
    fn replay_service_accepts_imported_run_without_local_repo_state() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let imported = tmp.path().join("run-imported");
        let replay = tmp.path().join("run-replay");
        fs::create_dir_all(&imported).expect("create imported");
        fs::create_dir_all(&replay).expect("create replay");
        write(
            &imported.join("manifest.json"),
            r#"{"status":"completed","run_metadata":{"submission_source":"import"}}"#,
        );
        write(
            &replay.join("manifest.json"),
            r#"{"status":"completed","run_metadata":{"submission_source":"manual"}}"#,
        );
        write(&imported.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&replay.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        let diff = run_diff_from_dirs(&imported, &replay).expect("build run diff");
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
    }

    #[test]
    fn replay_service_supports_older_run_manifest_versions_when_shape_is_compatible() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let old = tmp.path().join("run-old");
        let current = tmp.path().join("run-current");
        fs::create_dir_all(&old).expect("create old");
        fs::create_dir_all(&current).expect("create current");
        write(
            &old.join("manifest.json"),
            r#"{"manifest_version":"run-manifest/v0.1","status":"completed"}"#,
        );
        write(
            &current.join("manifest.json"),
            r#"{"manifest_version":"run-manifest/v0.1","status":"completed"}"#,
        );
        write(&old.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&current.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        let diff = run_diff_from_dirs(&old, &current).expect("build run diff");
        assert!(diff.replay_equivalence.equivalent);
    }

    #[test]
    fn replay_service_reports_failure_grouping_for_node_outcome_drift() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("nodes/n1")).expect("create nodes a");
        fs::create_dir_all(run_b.join("nodes/n1")).expect("create nodes b");
        write(&run_a.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed"}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("nodes/n1/trace.json"), r#"{"status":"success"}"#);
        write(&run_b.join("nodes/n1/trace.json"), r#"{"status":"failed"}"#);

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(!diff.replay_equivalence.equivalent);
        assert_eq!(diff.replay_equivalence.cause_groups.get("node_outcomes").copied(), Some(1));
    }

    #[test]
    fn replay_service_reports_downgrade_fidelity_when_graph_fingerprint_differs() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let imported = tmp.path().join("run-imported");
        let replay = tmp.path().join("run-replay");
        fs::create_dir_all(&imported).expect("create imported");
        fs::create_dir_all(&replay).expect("create replay");
        write(
            &imported.join("manifest.json"),
            r#"{"status":"completed","run_metadata":{"submission_source":"import"}}"#,
        );
        write(
            &replay.join("manifest.json"),
            r#"{"status":"completed","run_metadata":{"submission_source":"manual"}}"#,
        );
        write(&imported.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-import"}"#);
        write(&replay.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-local"}"#);
        let diff = run_diff_from_dirs(&imported, &replay).expect("build run diff");
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff
            .replay_equivalence
            .reasons
            .iter()
            .any(|reason| reason.contains("graph fingerprint differs")));
    }

    #[test]
    fn replay_service_classifies_missing_evidence_categories() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run = tmp.path().join("run-gap");
        fs::create_dir_all(run.join("nodes/n1")).expect("create nodes");
        write(&run.join("manifest.json"), r#"{"status":"completed"}"#);
        write(
            &run.join("nodes/n1/trace.json"),
            r#"{"status":"success","adapter_id":"","adapter_version":"1"}"#,
        );

        let gaps = replay_evidence_gaps(&run);
        assert!(gaps.contains(&"missing_graph".to_string()));
        assert!(gaps.contains(&"missing_policy".to_string()));
        assert!(gaps.contains(&"missing_adapter_identity".to_string()));
        assert!(gaps.contains(&"missing_artifact_hash".to_string()));
    }
}
