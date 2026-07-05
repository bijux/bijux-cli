use crate::commands::DagCli;
use crate::emit_json;
use crate::replay_service;
use crate::routes::renderer::print_pretty_json;
use crate::{read_file, ExitCode};
use std::path::Path;

pub(crate) fn why_rerun_payload(
    run_a: &Path,
    run_b: &Path,
    node: Option<&str>,
) -> Result<serde_json::Value, ExitCode> {
    replay_service::why_rerun_payload(run_a, run_b, node)
}

pub(crate) fn handle_why_rerun_command(
    cli: &DagCli,
    run_a: &Path,
    run_b: &Path,
    node: Option<&str>,
) -> Result<ExitCode, ExitCode> {
    let payload = why_rerun_payload(run_a, run_b, node)?;
    if cli.json {
        return emit_json(cli, "dag.why-rerun", true, payload, Vec::new(), ExitCode::SUCCESS);
    }
    print_pretty_json(&payload);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn trace_artifact_payload(
    run_dir: &Path,
    artifact_id: &str,
) -> Result<serde_json::Value, ExitCode> {
    let details = crate::inspect_artifact(run_dir, artifact_id)?;
    Ok(serde_json::json!({
        "artifact_id": details["artifact_id"],
        "path": details["path"],
        "provenance": details["provenance"],
        "lineage": details["lineage"]
    }))
}

pub(crate) fn handle_trace_artifact_command(
    cli: &DagCli,
    run_dir: &Path,
    artifact_id: &str,
) -> Result<ExitCode, ExitCode> {
    let payload = trace_artifact_payload(run_dir, artifact_id)?;
    if cli.json {
        return emit_json(cli, "dag.trace-artifact", true, payload, Vec::new(), ExitCode::SUCCESS);
    }
    print_pretty_json(&payload);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn trace_node_payload(
    run_dir: &Path,
    node_id: &str,
) -> Result<serde_json::Value, ExitCode> {
    let snapshot = read_file(&run_dir.join("graph.snapshot.json")).and_then(|raw| {
        serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| ExitCode::from(3))
    })?;
    let node = snapshot
        .get("graph")
        .and_then(|value| value.get("nodes"))
        .and_then(|value| value.as_array())
        .and_then(|nodes| {
            nodes.iter().find(|candidate| {
                candidate.get("id").and_then(|value| value.as_str()) == Some(node_id)
            })
        })
        .cloned()
        .ok_or(ExitCode::from(3))?;
    let trace =
        read_file(&run_dir.join("nodes").join(node_id).join("trace.json")).and_then(|raw| {
            serde_json::from_str::<serde_json::Value>(&raw).map_err(|_| ExitCode::from(3))
        })?;
    let outputs_index =
        read_file(&run_dir.join("nodes").join(node_id).join("outputs").join("index.json"))
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let deps = snapshot
        .get("graph")
        .and_then(|value| value.get("edges"))
        .and_then(|value| value.as_array())
        .map(|edges| {
            edges
                .iter()
                .filter_map(|edge| {
                    let to_node = edge
                        .get("to")
                        .and_then(|value| value.get("node_id"))
                        .and_then(|value| value.as_str())?;
                    if to_node != node_id {
                        return None;
                    }
                    edge.get("from")
                        .and_then(|value| value.get("node_id"))
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(serde_json::json!({
        "node_id": node_id,
        "kind": node.get("kind").cloned().unwrap_or(serde_json::Value::Null),
        "deps": deps,
        "outputs": node.get("outputs").cloned().unwrap_or(serde_json::Value::Null),
        "effects": node.get("effects").cloned().unwrap_or_else(|| serde_json::json!([])),
        "trace": trace,
        "outputs_index": outputs_index,
    }))
}

pub(crate) fn handle_trace_node_command(
    cli: &DagCli,
    run_dir: &Path,
    node_id: &str,
) -> Result<ExitCode, ExitCode> {
    let payload = trace_node_payload(run_dir, node_id)?;
    if cli.json {
        return emit_json(cli, "dag.trace-node", true, payload, Vec::new(), ExitCode::SUCCESS);
    }
    print_pretty_json(&payload);
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{
        handle_trace_artifact_command, handle_trace_node_command, handle_why_rerun_command,
        trace_artifact_payload, trace_node_payload, why_rerun_payload,
    };
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use serde_json::json;
    use serde_json::Value;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
    }

    #[test]
    fn why_rerun_route_rejects_missing_run_dir_without_panic() {
        let cli = quiet_json_cli();
        let result =
            handle_why_rerun_command(&cli, Path::new("/missing/a"), Path::new("/missing/b"), None);
        assert!(result.is_err());
    }

    #[test]
    fn trace_artifact_route_rejects_missing_run_dir_without_panic() {
        let cli = DagCli {
            json: true,
            quiet: true,
            command: Commands::TraceArtifact {
                run_dir: PathBuf::from("/missing/run"),
                artifact_id: "n1:out".to_string(),
            },
        };
        let code =
            handle_trace_artifact_command(&cli, Path::new("/missing/run"), "n1:out").unwrap_err();
        assert_eq!(code, ExitCode::from(3));
    }

    fn write_diff_ready_runs() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().expect("tmp");
        let root = dir.path().join("runs");
        let run_a = root.join("run-a");
        let run_b = root.join("run-b");
        for run in [&run_a, &run_b] {
            fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir");
            fs::create_dir_all(run.join("outputs")).expect("mkdir");
            fs::write(run.join("nodes/extract/outputs/data.txt"), b"x").expect("payload");
            fs::write(
                run.join("manifest.json"),
                serde_json::to_vec_pretty(&json!({
                    "manifest_version":"run-manifest/v0.1",
                    "run_id": run.file_name().unwrap().to_string_lossy(),
                    "created_unix_ms":1,"started_unix_ms":1,"finished_unix_ms":2,
                    "graph_snapshot":"graph.snapshot.json","status":"success","spec":"bijux-dag/v0.1",
                    "graph_fingerprint":"g1","tool_version":"0.1.0","jobs":1,
                    "adapters":[],"outputs":[],"node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
                    "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
                }))
                .expect("manifest"),
            )
            .expect("write manifest");
            fs::write(
                run.join("graph.snapshot.json"),
                serde_json::to_vec_pretty(&json!({
                    "graph":{"spec":"bijux-dag/v0.1","meta":{"name":"x","owners":[],"tags":[]},"nodes":[{"id":"extract","kind":"const","inputs":[],"outputs":[{"name":"out","path":"extract/out"}],"params":{"value":"x"}}],"edges":[]},
                    "graph_fingerprint":"g1"
                }))
                .expect("snapshot"),
            )
            .expect("write snap");
            fs::write(
                run.join("outputs/index.json"),
                serde_json::to_vec_pretty(&json!({"files":[{"node_id":"extract","node_fingerprint":"fp1","name":"out","kind":"file","media_type":"text/plain","size_bytes":1,"sha256":"2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881","path":"nodes/extract/outputs/data.txt"}]}))
                    .expect("index"),
            )
            .expect("write index");
            fs::write(
                run.join("nodes/extract/outputs/index.json"),
                serde_json::to_vec_pretty(&json!({"files":[{"node_id":"extract","node_fingerprint":"fp1","name":"out","kind":"file","media_type":"text/plain","size_bytes":1,"sha256":"2d711642b726b04401627ca9fbac32f5c8530fb1903cc4db02258717921a4881","path":"nodes/extract/outputs/data.txt"}]}))
                    .expect("node index"),
            )
            .expect("write node index");
            fs::write(
                run.join("nodes/extract/trace.json"),
                serde_json::to_vec_pretty(&json!({"status":"success","attempt":1})).expect("trace"),
            )
            .expect("write trace");
        }
        (dir, run_a, run_b)
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).expect("read json")).expect("parse json")
    }

    fn write_json(path: &Path, value: &Value) {
        fs::write(path, serde_json::to_vec_pretty(value).expect("encode json"))
            .expect("write json");
    }

    #[test]
    fn diagnostics_success_paths_return_payloads() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let why = why_rerun_payload(&run_a, &run_b, None).expect("why rerun");
        assert!(why.get("root_cause_summary").is_some());
        let trace = trace_artifact_payload(&run_a, "extract:data.txt").expect("trace artifact");
        assert!(trace["artifact_id"]
            .as_str()
            .expect("canonical artifact id")
            .starts_with("run=run-a;node=extract;path=nodes/extract/outputs/data.txt;sha256="));
        let node = trace_node_payload(&run_a, "extract").expect("trace node");
        assert_eq!(node["node_id"], "extract");
    }

    #[test]
    fn diagnostics_route_handlers_support_success_paths() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let cli = quiet_json_cli();
        let why = handle_why_rerun_command(&cli, &run_a, &run_b, None).expect("handle why rerun");
        assert_eq!(why, ExitCode::SUCCESS);
        let trace = handle_trace_artifact_command(&cli, &run_a, "extract:data.txt")
            .expect("handle trace artifact");
        assert_eq!(trace, ExitCode::SUCCESS);
        let node = handle_trace_node_command(&cli, &run_a, "extract").expect("handle trace node");
        assert_eq!(node, ExitCode::SUCCESS);
    }

    #[test]
    fn diagnostics_routes_do_not_panic_on_malformed_inputs() {
        let cli = quiet_json_cli();
        let why = std::panic::catch_unwind(|| {
            handle_why_rerun_command(&cli, Path::new("/missing/a"), Path::new("/missing/b"), None)
        });
        let trace = std::panic::catch_unwind(|| {
            handle_trace_artifact_command(&cli, Path::new("/missing/run"), "broken")
        });
        assert!(why.is_ok());
        assert!(trace.is_ok());
    }

    #[test]
    fn why_rerun_reports_graph_drift_group() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let mut snap = read_json(&run_b.join("graph.snapshot.json"));
        snap["graph"]["nodes"][0]["params"]["value"] = json!("changed");
        snap["graph_fingerprint"] = json!("g2");
        write_json(&run_b.join("graph.snapshot.json"), &snap);

        let mut manifest = read_json(&run_b.join("manifest.json"));
        manifest["graph_fingerprint"] = json!("g2");
        write_json(&run_b.join("manifest.json"), &manifest);

        let payload = why_rerun_payload(&run_a, &run_b, None).expect("why rerun");
        assert_eq!(payload["equivalent"], false);
        assert!(payload["cause_groups"].get("graph_semantics").is_some());
    }

    #[test]
    fn why_rerun_reports_environment_drift_group() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let mut manifest = read_json(&run_b.join("manifest.json"));
        manifest["policy"]["deny_env"] = json!(false);
        write_json(&run_b.join("manifest.json"), &manifest);

        let payload = why_rerun_payload(&run_a, &run_b, None).expect("why rerun");
        assert_eq!(payload["equivalent"], false);
        assert!(payload["cause_groups"].get("manifest_drift").is_some());
    }

    #[test]
    fn why_rerun_reports_artifact_drift_group() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let mut outputs = read_json(&run_b.join("nodes/extract/outputs/index.json"));
        outputs["files"][0]["sha256"] =
            json!("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
        write_json(&run_b.join("nodes/extract/outputs/index.json"), &outputs);

        let payload = why_rerun_payload(&run_a, &run_b, None).expect("why rerun");
        assert_eq!(payload["equivalent"], false);
        assert!(payload["cause_groups"].get("artifact_payload").is_some());
    }

    #[test]
    fn why_rerun_reports_replay_ancestry_drift_group() {
        let (_tmp, run_a, run_b) = write_diff_ready_runs();
        let mut manifest_a = read_json(&run_a.join("manifest.json"));
        manifest_a["run_metadata"] = json!({
            "parent_run_id":"run-parent-a",
            "source_run_id":"run-source-a",
            "submission_source":"replay",
            "trigger_source":"cli"
        });
        write_json(&run_a.join("manifest.json"), &manifest_a);
        let mut manifest = read_json(&run_b.join("manifest.json"));
        manifest["run_metadata"] = json!({
            "parent_run_id":"run-parent-b",
            "source_run_id":"run-source-b",
            "submission_source":"replay",
            "trigger_source":"cli"
        });
        manifest["jobs"] = json!(2);
        write_json(&run_b.join("manifest.json"), &manifest);

        let payload = why_rerun_payload(&run_a, &run_b, None).expect("why rerun");
        assert_eq!(payload["equivalent"], false);
        assert!(payload["cause_groups"].get("manifest_drift").is_some());
    }
}
