use crate::commands::DagCli;
use crate::routes::path_resolution::{manifest_path, node_outputs_index_path, node_trace_path};
use crate::routes::run_lookup::read_manifest_json;
use crate::run_data::{load_snapshot, read_node_traces};
use crate::{emit_json, read_file, ExitCode};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn concise_explain_human(
    status: &Value,
    graph_fp: &Value,
    counts: &Value,
    failed: &[String],
) -> String {
    format!(
        "status: {status}\ngraph_fingerprint: {graph_fp}\nnode_counts: {counts}\nfailed_nodes: {failed:?}"
    )
}

pub(crate) fn handle_explain_command(
    cli: &DagCli,
    run_dir: &Path,
    node: &Option<String>,
) -> Result<ExitCode, ExitCode> {
    let manifest = read_file(&manifest_path(run_dir))?;
    if let Some(node_id) = node.as_ref() {
        let snapshot = load_snapshot(run_dir)?;
        let trace = read_file(&node_trace_path(run_dir, node_id))?;
        let node_info =
            snapshot.graph.nodes.iter().find(|n| n.id == *node_id).ok_or(ExitCode::from(3))?;
        let deps = snapshot
            .graph
            .edges
            .iter()
            .filter(|e| e.to.node_id == *node_id)
            .map(|e| e.from.node_id.clone())
            .collect::<Vec<_>>();
        let outputs_index = read_file(&node_outputs_index_path(run_dir, node_id)).ok();
        let resolved_params =
            read_file(&run_dir.join("nodes").join(node_id).join("resolved_params.json")).ok();
        let outputs = node_info.outputs.clone();
        let inputs = node_info.inputs.clone();
        if cli.json {
            let data = json!({
                "manifest": serde_json::from_str::<serde_json::Value>(&manifest).ok(),
                "node": node_id,
                "deps": deps,
                "inputs": inputs,
                "outputs": outputs,
                "effects": node_info.effects,
                "env_allowlist": node_info.env_allowlist,
                "outputs_index": outputs_index.and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok()),
                "resolved_params": resolved_params.and_then(|v| serde_json::from_str::<serde_json::Value>(&v).ok()),
                "trace": serde_json::from_str::<serde_json::Value>(&trace).ok(),
                "fingerprint": snapshot.graph.node_fingerprint(node_info).ok(),
            });
            return emit_json(cli, "dag.explain", true, data, Vec::new(), ExitCode::SUCCESS);
        } else {
            println!("node: {}", node_id);
            println!("deps: {:?}", deps);
            println!("inputs: {:?}", inputs);
            println!("outputs: {:?}", outputs);
            println!("effects: {:?}", node_info.effects);
            println!("env_allowlist: {:?}", node_info.env_allowlist);
            if let Some(r) = resolved_params {
                println!("resolved_params:\n{}", r);
            }
            if let Some(o) = outputs_index {
                println!("outputs_index:\n{}", o);
            }
            println!("fingerprint: {:?}", snapshot.graph.node_fingerprint(node_info).ok());
            println!("trace:\n{}", trace);
        }
    } else if cli.json {
        let m: serde_json::Value = read_manifest_json(run_dir).unwrap_or_default();
        let status = m.get("status").cloned().unwrap_or_default();
        let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
        let counts = m.get("node_counts").cloned().unwrap_or_default();
        let nodes = read_node_traces(run_dir).unwrap_or_default();
        let failed: Vec<String> = nodes
            .iter()
            .filter_map(|(id, v)| {
                if v.get("status") == Some(&serde_json::Value::String("failed".to_string())) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        let data = json!({
            "status": status,
            "graph_fingerprint": graph_fp,
            "node_counts": counts,
            "failed_nodes": failed,
        });
        return emit_json(cli, "dag.explain", true, data, Vec::new(), ExitCode::SUCCESS);
    } else {
        let m: serde_json::Value = read_manifest_json(run_dir).unwrap_or_default();
        let status = m.get("status").cloned().unwrap_or_default();
        let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
        let counts = m.get("node_counts").cloned().unwrap_or_default();
        let nodes = read_node_traces(run_dir).unwrap_or_default();
        let failed: Vec<String> = nodes
            .iter()
            .filter_map(|(id, v)| {
                if v.get("status") == Some(&serde_json::Value::String("failed".to_string())) {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();
        println!("{}", concise_explain_human(&status, &graph_fp, &counts, &failed));
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_node_command(
    cli: &DagCli,
    run_dir: &Path,
    node: &str,
) -> Result<ExitCode, ExitCode> {
    let trace = read_file(&node_trace_path(run_dir, node))?;
    let index = read_file(&node_outputs_index_path(run_dir, node))?;
    if cli.json {
        return emit_json(
            cli,
            "dag.node",
            true,
            json!({"trace": trace, "outputs": index}),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("trace:\n{}", trace);
    println!("outputs:\n{}", index);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_status_command(cli: &DagCli, run_dir: &Path) -> Result<ExitCode, ExitCode> {
    let manifest = read_file(&manifest_path(run_dir))?;
    let nodes_dir = run_dir.join("nodes");
    let manifest_json =
        serde_json::from_str::<Value>(&manifest).unwrap_or(Value::String(manifest.clone()));
    let mut statuses = Vec::new();
    if nodes_dir.exists() {
        for entry in fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))? {
            let entry = entry.map_err(|_| ExitCode::from(3))?;
            let trace_path = entry.path().join("trace.json");
            if trace_path.exists() {
                let t = read_file(&trace_path)?;
                statuses.push(serde_json::from_str::<Value>(&t).unwrap_or(Value::String(t)));
            }
        }
    }
    if cli.json {
        return emit_json(
            cli,
            "dag.status",
            true,
            json!({"manifest": manifest_json, "traces": statuses}),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    println!("manifest:\n{}", manifest);
    println!("traces: {}", statuses.len());
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{
        concise_explain_human, handle_explain_command, handle_node_command, handle_status_command,
    };
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use serde_json::json;
    use std::fs;
    use std::path::Path;

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
    }

    fn quiet_human_cli() -> DagCli {
        DagCli { json: false, quiet: true, command: Commands::Version }
    }

    fn write_run_fixture(imported: bool, malformed_manifest: bool) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tmp");
        let run = dir.path();
        fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir nodes");
        if malformed_manifest {
            fs::write(run.join("manifest.json"), b"{not-json").expect("write malformed manifest");
        } else {
            let mut manifest = json!({
                "manifest_version":"run-manifest/v0.1",
                "run_id":"run-1",
                "created_unix_ms":1,
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "graph_snapshot":"graph.snapshot.json",
                "status":"success",
                "spec":"bijux-dag/v0.1",
                "graph_fingerprint":"g1",
                "tool_version":"0.1.0",
                "jobs":1,
                "adapters":[],
                "outputs":[],
                "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
                "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
            });
            if imported {
                manifest["import_source"] = json!("bundle");
            }
            fs::write(
                run.join("manifest.json"),
                serde_json::to_vec_pretty(&manifest).expect("manifest"),
            )
            .expect("write manifest");
        }
        fs::write(
            run.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph":{"spec":"bijux-dag/v0.1","meta":{"name":"x","owners":[],"tags":[]},"nodes":[{"id":"extract","kind":"const","inputs":[],"outputs":[{"name":"out","path":"extract/out"}],"params":{"value":"x"}}],"edges":[]},
                "graph_fingerprint":"g1"
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(run.join("nodes/extract/trace.json"), b"{\"status\":\"success\"}")
            .expect("trace");
        fs::write(run.join("nodes/extract/outputs/index.json"), b"{\"files\":[]}").expect("index");
        dir
    }

    #[test]
    fn inspect_status_success_json_path() {
        let run = write_run_fixture(false, false);
        let cli = quiet_json_cli();
        let code = handle_status_command(&cli, run.path()).expect("status");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_explain_success_json_path() {
        let run = write_run_fixture(false, false);
        let cli = quiet_json_cli();
        let code = handle_explain_command(&cli, run.path(), &None).expect("explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_routes_handle_imported_run_paths() {
        let run = write_run_fixture(true, false);
        let cli = quiet_json_cli();
        let code = handle_explain_command(&cli, run.path(), &None).expect("imported explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn inspect_routes_handle_damaged_bundle_like_manifest_without_panic() {
        let run = write_run_fixture(false, true);
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| handle_explain_command(&cli, run.path(), &None));
        assert!(result.is_ok(), "inspect explain should not panic");
        assert!(result.expect("result").is_ok());
    }

    #[test]
    fn inspect_node_malformed_run_dir_returns_error() {
        let cli = quiet_json_cli();
        let code = handle_node_command(&cli, Path::new("/missing/run"), "extract").unwrap_err();
        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn inspect_human_paths_do_not_panic() {
        let run = write_run_fixture(false, false);
        let cli = quiet_human_cli();
        let explain = std::panic::catch_unwind(|| handle_explain_command(&cli, run.path(), &None));
        assert!(explain.is_ok());
        assert!(explain.expect("result").is_ok());
        let status = std::panic::catch_unwind(|| handle_status_command(&cli, run.path()));
        assert!(status.is_ok());
        assert!(status.expect("result").is_ok());
    }

    #[test]
    fn inspect_route_entrypoints_do_not_panic_on_missing_run_dir() {
        let cli = quiet_json_cli();
        let explain = std::panic::catch_unwind(|| {
            handle_explain_command(&cli, Path::new("/missing/run"), &None)
        });
        let node = std::panic::catch_unwind(|| {
            handle_node_command(&cli, Path::new("/missing/run"), "extract")
        });
        let status =
            std::panic::catch_unwind(|| handle_status_command(&cli, Path::new("/missing/run")));
        assert!(explain.is_ok());
        assert!(node.is_ok());
        assert!(status.is_ok());
    }

    #[test]
    fn inspect_concise_human_snapshot_is_stable() {
        let rendered = concise_explain_human(
            &json!("success"),
            &json!("g1"),
            &json!({"success":1,"failed":0,"skipped":0,"cached":0}),
            &["n1".to_string()],
        );
        let expected = "status: \"success\"\n\
graph_fingerprint: \"g1\"\n\
node_counts: {\"cached\":0,\"failed\":0,\"skipped\":0,\"success\":1}\n\
failed_nodes: [\"n1\"]";
        assert_eq!(rendered, expected);
    }
}
