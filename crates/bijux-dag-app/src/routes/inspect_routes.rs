use crate::commands::DagCli;
use crate::routes::path_resolution::{manifest_path, node_outputs_index_path, node_trace_path};
use crate::routes::run_lookup::read_manifest_json;
use crate::{emit_json, load_snapshot, read_file, read_node_traces, ExitCode};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub(crate) fn handle_explain_command(
    cli: &DagCli,
    run_dir: &Path,
    node: &Option<String>,
) -> Result<ExitCode, ExitCode> {
    let manifest = read_file(&manifest_path(run_dir))?;
    if let Some(node_id) = node.as_ref() {
        let snapshot = load_snapshot(run_dir)?;
        let trace = read_file(&node_trace_path(run_dir, node_id))?;
        let node_info = snapshot
            .graph
            .nodes
            .iter()
            .find(|n| n.id == *node_id)
            .ok_or(ExitCode::from(3))?;
        let deps = snapshot
            .graph
            .edges
            .iter()
            .filter(|e| e.to.node_id == *node_id)
            .map(|e| e.from.node_id.clone())
            .collect::<Vec<_>>();
        let outputs_index = read_file(&node_outputs_index_path(run_dir, node_id)).ok();
        let resolved_params = read_file(
            &run_dir
                .join("nodes")
                .join(node_id)
                .join("resolved_params.json"),
        )
        .ok();
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
            return emit_json(
                cli,
                "dag.explain",
                true,
                data,
                Vec::new(),
                ExitCode::SUCCESS,
            );
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
            println!(
                "fingerprint: {:?}",
                snapshot.graph.node_fingerprint(node_info).ok()
            );
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
        return emit_json(
            cli,
            "dag.explain",
            true,
            data,
            Vec::new(),
            ExitCode::SUCCESS,
        );
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
        println!("status: {}", status);
        println!("graph_fingerprint: {}", graph_fp);
        println!("node_counts: {}", counts);
        println!("failed_nodes: {:?}", failed);
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
