use crate::diff::{build_run_diff, RunDiff};
use bijux_dag_artifacts::OutputsIndex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
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
    use super::run_diff_from_dirs;
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
        write(
            &run_a.join("graph.snapshot.json"),
            r#"{"graph_fingerprint":"fp-1"}"#,
        );
        write(
            &run_b.join("graph.snapshot.json"),
            r#"{"graph_fingerprint":"fp-1"}"#,
        );

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
        write(
            &run_a.join("graph.snapshot.json"),
            r#"{"graph_fingerprint":"fp-1"}"#,
        );
        write(
            &run_b.join("graph.snapshot.json"),
            r#"{"graph_fingerprint":"fp-2"}"#,
        );

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff
            .replay_equivalence
            .reasons
            .iter()
            .any(|reason| { reason.contains("graph fingerprint differs") }));
    }
}
