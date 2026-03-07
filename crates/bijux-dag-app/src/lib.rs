mod cache;
mod commands;
mod diff;
mod explain;
mod format;
mod graph;
mod migrate;
mod read;
mod replay;
mod write;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_artifacts::{OutputsIndex, RunOutputsIndex};
use bijux_dag_core::{parse_graph_strict, Graph, GraphError, Severity, SPEC_VERSION};
use bijux_dag_runtime::{
    adapter_registry_dump, registered_adapters, CacheMode, MaterializeMode, Runtime, RuntimeConfig, Selector, SelectorSet,
};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use commands::{
    AdaptersCommands, CacheCommands, CacheModeArg, Commands, DagCli, GraphFormatArg,
    MaterializeModeArg, MigrateCommands,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tar::{Archive, Builder};

pub fn dag_command() -> clap::Command {
    DagCli::command().name("dag")
}

pub fn dag_run(matches: &ArgMatches) -> Result<ExitCode, ExitCode> {
    let cli = DagCli::from_arg_matches(matches).map_err(|_| ExitCode::from(2))?;
    run(cli)
}


#[derive(Debug, Serialize)]
struct LintDiagnostic {
    code: String,
    message: String,
    path: String,
    hint: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonEnvelope {
    ok: bool,
    command: String,
    data: Value,
    diagnostics: Vec<Value>,
}

fn emit_json(
    cli: &DagCli,
    command: &str,
    ok: bool,
    data: Value,
    diagnostics: Vec<Value>,
    code: ExitCode,
) -> Result<ExitCode, ExitCode> {
    if !cli.quiet {
        let envelope = JsonEnvelope {
            ok,
            command: command.to_string(),
            data,
            diagnostics,
        };
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    }
    if ok {
        Ok(code)
    } else {
        Err(code)
    }
}

fn run(cli: DagCli) -> Result<ExitCode, ExitCode> {
    match &cli.command {
        Commands::Init { dir } => {
            let base = dir.clone().unwrap_or_else(|| PathBuf::from("."));
            fs::create_dir_all(&base).map_err(|_| ExitCode::from(3))?;
            let dag_path = base.join("dag.json");
            if dag_path.exists() {
                return Err(ExitCode::from(3));
            }
            let runs_dir = base.join("runs");
            fs::create_dir_all(&runs_dir).map_err(|_| ExitCode::from(3))?;
            let docs_spec_dir = base.join("docs").join("spec");
            fs::create_dir_all(&docs_spec_dir).ok();
            let dag = json!({
              "spec": SPEC_VERSION,
              "meta": {
                "name": "hello-bijux-dag",
                "description": "Starter Bijux DAG",
                "owners": [],
                "tags": []
              },
              "nodes": [
                {
                  "id": "const1",
                  "kind": "const",
                  "inputs": [],
                  "outputs": [{"name": "out", "path": "out"}],
                  "params": {"value": "hello"}
                },
                {
                  "id": "echo",
                  "kind": "shell",
                  "inputs": ["in"],
                  "outputs": [{"name": "out", "path": "out"}],
                  "params": {"argv": ["/bin/sh","-c","cat ../inputs/const1/in > ../outputs/out"]},
                  "effects": ["filesystem"]
                }
              ],
              "edges": [
                {"from": {"node_id": "const1", "port": "out"}, "to": {"node_id": "echo", "port": "in"}}
              ]
            });
            fs::write(&dag_path, serde_json::to_vec_pretty(&dag).unwrap())
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.init",
                    true,
                    json!({"dag": dag_path, "runs": runs_dir}),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else if !cli.quiet {
                println!("created {}", dag_path.display());
                println!("created {}", runs_dir.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Validate {
            dag,
            strict,
            print_fingerprints,
            explain,
        } => {
            let strict = *strict;
            let print_fingerprints = *print_fingerprints;
            let explain = *explain;
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let diags = graph.validate_with_warnings();
            let has_errors = diags.iter().any(|d| d.severity == Severity::Error);
            let has_warnings = diags.iter().any(|d| d.severity == Severity::Warning);
            let fail = has_errors || (strict && has_warnings);

            let diagnostics: Vec<Value> = diags
                .iter()
                .map(|d| serde_json::to_value(d).unwrap())
                .collect();
            let mut data = json!({});
            if print_fingerprints || explain {
                data["graph_fingerprint"] = json!(graph.graph_fingerprint().unwrap());
                let mut nodes = serde_json::Map::new();
                let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
                for n in &graph.nodes {
                    let fp = resolved
                        .as_ref()
                        .and_then(|m| m.get(&n.id))
                        .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                        .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
                    nodes.insert(n.id.clone(), json!(fp));
                }
                data["node_fingerprints"] = json!(nodes);
            }
            if explain {
                let canonical = graph.canonicalize();
                let order = canonical
                    .nodes
                    .iter()
                    .map(|n| n.id.clone())
                    .collect::<Vec<_>>();
                data["canonical_order"] = json!(order);
                data["resolved_params"] = json!(graph
                    .resolve_graph()
                    .map(|g| g.resolved_params)
                    .unwrap_or_default());
            }
            if cli.json {
                let code = if fail {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                };
                return emit_json(&cli, "dag.validate", !fail, data, diagnostics, code);
            } else if !cli.quiet {
                for d in &diags {
                    println!("{} {} {}", d.code, d.path, d.message);
                }
                if print_fingerprints {
                    println!("graph_fingerprint={}", graph.graph_fingerprint().unwrap());
                    let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
                    for n in &graph.nodes {
                        let fp = resolved
                            .as_ref()
                            .and_then(|m| m.get(&n.id))
                            .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                            .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
                        println!("node_fingerprint {}={}", n.id, fp);
                    }
                }
                if explain {
                    let canonical = graph.canonicalize();
                    let order = canonical
                        .nodes
                        .iter()
                        .map(|n| n.id.clone())
                        .collect::<Vec<_>>();
                    println!("canonical_order: {:?}", order);
                    println!(
                        "resolved_params: {}",
                        serde_json::to_string_pretty(
                            &graph
                                .resolve_graph()
                                .map(|g| g.resolved_params)
                                .unwrap_or_default()
                        )
                        .unwrap()
                    );
                    println!("graph_fingerprint={}", graph.graph_fingerprint().unwrap());
                    let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
                    for n in &graph.nodes {
                        let fp = resolved
                            .as_ref()
                            .and_then(|m| m.get(&n.id))
                            .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                            .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
                        println!("node_fingerprint {}={}", n.id, fp);
                    }
                }
            }
            if fail {
                return Err(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Canonicalize { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let json = graph.to_canonical_json().map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.canonicalize",
                    true,
                    json!({ "canonical": json }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", json);
            Ok(ExitCode::SUCCESS)
        }
        Commands::Lint { dag, strict } => {
            let strict = *strict;
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let lint = lint_graph(&graph);
            let has_warnings = !lint.is_empty();
            if cli.json {
                let diagnostics: Vec<Value> = lint
                    .iter()
                    .map(|d| serde_json::to_value(d).unwrap())
                    .collect();
                let code = if strict && has_warnings {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                };
                return emit_json(
                    &cli,
                    "dag.lint",
                    !(strict && has_warnings),
                    json!({}),
                    diagnostics,
                    code,
                );
            } else {
                for warn in &lint {
                    println!("WARN {} {} {}", warn.code, warn.path, warn.message);
                }
            }
            if strict && has_warnings {
                return Err(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fingerprint { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let fp = graph.graph_fingerprint().map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.fingerprint",
                    true,
                    json!({"graph": fp}),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                println!("{}", fp);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Graph { dag, format } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            match format {
                GraphFormatArg::Dot => {
                    let dot = graph_to_dot(&graph);
                    if cli.json {
                        return emit_json(
                            &cli,
                            "dag.graph",
                            true,
                            json!({ "dot": dot }),
                            Vec::new(),
                            ExitCode::SUCCESS,
                        );
                    }
                    println!("{}", dot);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Replay {
            run_dir,
            out,
            reuse_cache,
            cache,
            jobs,
            run_id,
            cpu_budget,
            deny_network,
            deny_env,
            deny_clock,
            clean_env,
            hermetic,
            select,
            exclude,
            materialize_inputs,
            remote_cache_dir,
        } => {
            let snapshot = load_snapshot(run_dir)?;
            let runtime = Runtime::new();
            let cache_mode = match *cache {
                CacheModeArg::Off => {
                    if *reuse_cache {
                        CacheMode::Read
                    } else {
                        CacheMode::Off
                    }
                }
                CacheModeArg::Read => CacheMode::Read,
                CacheModeArg::Readwrite => CacheMode::ReadWrite,
            };
            let mut deny_network = *deny_network;
            let mut deny_clock = *deny_clock;
            let deny_env = *deny_env;
            let clean_env_flag = *clean_env;
            let mut clean_env = clean_env_flag;
            if !clean_env_flag {
                clean_env = true;
            }
            if *hermetic {
                deny_network = true;
                deny_clock = true;
                clean_env = true;
            }
            let selectors = parse_selectors(select, exclude)?;
            let options = RuntimeConfig {
                jobs: *jobs,
                cpu_budget: *cpu_budget,
                run_timeout_ms: None,
                node_timeout_ms: None,
                materialize_inputs: map_materialize_mode(*materialize_inputs),
                cache_mode,
                cache_dir: None,
                remote_cache_dir: remote_cache_dir.clone(),
                run_id: run_id.clone(),
                latest_symlink: None,
                policy: bijux_dag_runtime::PolicyConfig {
                    deny_network,
                    deny_env,
                    deny_clock,
                    clean_env,
                },
                selectors,
            };
            let run_path = runtime
                .run(&snapshot.graph, out, options)
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.replay",
                    true,
                    json!({ "run_dir": run_path }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if !cli.quiet {
                println!("run dir: {}", run_path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Diff {
            run_a,
            run_b,
            explain,
        } => {
            let explain = *explain;
            let manifest_a = read_file(&run_a.join("manifest.json"))?;
            let manifest_b = read_file(&run_b.join("manifest.json"))?;
            let snap_a = load_snapshot(run_a)?;
            let snap_b = load_snapshot(run_b)?;
            let nodes_a = read_node_traces(run_a)?;
            let nodes_b = read_node_traces(run_b)?;
            let outputs_a = read_outputs_indexes(run_a)?;
            let outputs_b = read_outputs_indexes(run_b)?;
            let diff = diff::build_run_diff(
                serde_json::from_str(&manifest_a).unwrap_or_default(),
                serde_json::from_str(&manifest_b).unwrap_or_default(),
                snap_a.graph_fingerprint,
                snap_b.graph_fingerprint,
                &nodes_a,
                &nodes_b,
                &outputs_a,
                &outputs_b,
            );
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.diff",
                    true,
                    serde_json::to_value(&diff).unwrap(),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                print_human_diff(&serde_json::to_value(&diff).unwrap());
                if explain {
                    println!("explain: graph fingerprint change implies cache invalidation");
                    println!("explain: node fingerprint changes indicate recomputation scope");
                    println!(
                        "replay_equivalent: {}",
                        diff.replay_equivalence.equivalent
                    );
                    if !diff.replay_equivalence.reasons.is_empty() {
                        println!(
                            "replay_difference_reasons: {:?}",
                            diff.replay_equivalence.reasons
                        );
                    }
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Run {
            dag,
            out,
            run_id,
            latest,
            jobs,
            cpu_budget,
            node_timeout_ms,
            run_timeout_ms,
            deny_network,
            deny_env,
            deny_clock,
            clean_env,
            hermetic,
            select,
            exclude,
            materialize_inputs,
            cache,
            cache_dir,
            remote_cache_dir,
        } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let runtime = Runtime::new();
            let mut deny_network = *deny_network;
            let mut deny_clock = *deny_clock;
            let deny_env = *deny_env;
            let clean_env_flag = *clean_env;
            let mut clean_env = clean_env_flag;
            if !clean_env_flag {
                clean_env = true;
            }
            if *hermetic {
                deny_network = true;
                deny_clock = true;
                clean_env = true;
            }
            let selectors = parse_selectors(select, exclude)?;
            let options = RuntimeConfig {
                jobs: *jobs,
                cpu_budget: *cpu_budget,
                run_timeout_ms: *run_timeout_ms,
                node_timeout_ms: *node_timeout_ms,
                materialize_inputs: map_materialize_mode(*materialize_inputs),
                cache_mode: match *cache {
                    CacheModeArg::Off => CacheMode::Off,
                    CacheModeArg::Read => CacheMode::Read,
                    CacheModeArg::Readwrite => CacheMode::ReadWrite,
                },
                cache_dir: cache_dir.clone(),
                remote_cache_dir: remote_cache_dir.clone(),
                run_id: run_id.clone(),
                latest_symlink: latest.clone(),
                policy: bijux_dag_runtime::PolicyConfig {
                    deny_network,
                    deny_env,
                    deny_clock,
                    clean_env,
                },
                selectors,
            };
            let run_path = runtime
                .run(&graph, out, options)
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.run",
                    true,
                    json!({ "run_dir": run_path }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if !cli.quiet {
                println!("run dir: {}", run_path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Explain { run_dir, node } => {
            let manifest_path = run_dir.join("manifest.json");
            let manifest = read_file(&manifest_path)?;
            if let Some(node_id) = node.as_ref() {
                let snapshot = load_snapshot(run_dir)?;
                let trace = read_file(&run_dir.join("nodes").join(node_id).join("trace.json"))?;
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
                let outputs_index = read_file(
                    &run_dir
                        .join("nodes")
                        .join(node_id)
                        .join("outputs")
                        .join("index.json"),
                )
                .ok();
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
                        &cli,
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
                let m: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
                let status = m.get("status").cloned().unwrap_or_default();
                let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
                let counts = m.get("node_counts").cloned().unwrap_or_default();
                let nodes = read_node_traces(run_dir).unwrap_or_default();
                let failed: Vec<String> = nodes
                    .iter()
                    .filter_map(|(id, v)| {
                        if v.get("status") == Some(&serde_json::Value::String("failed".to_string()))
                        {
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
                    &cli,
                    "dag.explain",
                    true,
                    data,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                let m: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
                let status = m.get("status").cloned().unwrap_or_default();
                let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
                let counts = m.get("node_counts").cloned().unwrap_or_default();
                let nodes = read_node_traces(run_dir).unwrap_or_default();
                let failed: Vec<String> = nodes
                    .iter()
                    .filter_map(|(id, v)| {
                        if v.get("status") == Some(&serde_json::Value::String("failed".to_string()))
                        {
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
        Commands::Node { run_dir, id: node } => {
            let trace = read_file(&run_dir.join("nodes").join(node).join("trace.json"))?;
            let index = read_file(
                &run_dir
                    .join("nodes")
                    .join(node)
                    .join("outputs")
                    .join("index.json"),
            )?;
            if cli.json {
                return emit_json(
                    &cli,
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
        Commands::Status { run_dir } => {
            let manifest = read_file(&run_dir.join("manifest.json"))?;
            let nodes_dir = run_dir.join("nodes");
            let mut statuses = Vec::new();
            if nodes_dir.exists() {
                for entry in fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))? {
                    let entry = entry.map_err(|_| ExitCode::from(3))?;
                    let trace_path = entry.path().join("trace.json");
                    if trace_path.exists() {
                        let t = read_file(&trace_path)?;
                        statuses.push(t);
                    }
                }
            }
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.status",
                    true,
                    json!({"manifest": manifest, "traces": statuses}),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                println!("manifest:\n{}", manifest);
                println!("traces: {}", statuses.len());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Verify { run_dir, deep } => {
            let report = verify_run(run_dir, *deep)?;
            let ok = report
                .get("status")
                .and_then(|v| v.as_str())
                .map(|v| v == "ok")
                .unwrap_or(false);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.verify",
                    ok,
                    report,
                    Vec::new(),
                    if ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
                );
            } else {
                println!("status: {}", report["status"]);
            }
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Doctor => {
            let report = doctor_report()?;
            let ok = report
                .get("status")
                .and_then(|v| v.as_str())
                .map(|v| v == "ok")
                .unwrap_or(false);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.doctor",
                    ok,
                    report,
                    Vec::new(),
                    if ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
                );
            } else {
                println!("status: {}", report["status"]);
            }
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Migrate { command } => {
            let msg = match command {
                MigrateCommands::Dag { file, from, to } => migrate_dag(file, from, to)?,
                MigrateCommands::Run { run_dir, from, to } => migrate_run(run_dir, from, to)?,
            };
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.migrate",
                    true,
                    json!({ "message": msg }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else if !cli.quiet {
                println!("{}", msg);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Compat => {
            let report = run_compat_suite()?;
            let ok = report
                .get("status")
                .and_then(|v| v.as_str())
                .map(|v| v == "ok")
                .unwrap_or(false);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.compat",
                    ok,
                    report,
                    Vec::new(),
                    if ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
                );
            } else {
                println!("status: {}", report["status"]);
            }
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Cache { command } => match command {
            CacheCommands::Ls { cache_dir } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let mut entries_vec = Vec::new();
                if dir.exists() {
                    for entry in fs::read_dir(dir).map_err(|_| ExitCode::from(3))? {
                        let entry = entry.map_err(|_| ExitCode::from(3))?;
                        let name = entry.file_name().to_string_lossy().to_string();
                        if cli.json {
                            entries_vec.push(name);
                        } else {
                            println!("{}", name);
                        }
                    }
                }
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.ls",
                        true,
                        json!({ "entries": entries_vec }),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Pack {
                node_fp,
                out,
                cache_dir,
            } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let entry = dir.join(node_fp);
                if !entry.exists() {
                    return Err(ExitCode::from(3));
                }
                pack_cache_entry(&entry, out)?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.pack",
                        true,
                        json!({ "pack": out }),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                } else if !cli.quiet {
                    println!("pack: {}", out.display());
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Unpack { pack, cache_dir } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                unpack_cache_entry(pack, &dir)?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.unpack",
                        true,
                        json!({ "pack": pack }),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                } else if !cli.quiet {
                    println!("unpacked: {}", pack.display());
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Gc { cache_dir } => {
                let _dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.gc",
                        true,
                        json!({ "status": "stub" }),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("cache gc stub");
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Verify { cache_dir, remote } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let report = verify_cache_dirs(&dir, remote.as_ref().map(|v| v.as_path()))?;
                let corrupt = report["corrupt_total"].as_u64().unwrap_or(0);
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.verify",
                        corrupt == 0,
                        report,
                        Vec::new(),
                        if corrupt == 0 {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from(3)
                        },
                    );
                } else {
                    println!("local_checked: {}", report["local"]["checked"]);
                    println!("local_corrupt: {}", report["local"]["corrupt"]);
                    if let Some(keys) = report["local"]["corrupt_keys"].as_array() {
                        if !keys.is_empty() {
                            println!("local_corrupt_keys: {}", report["local"]["corrupt_keys"]);
                        }
                    }
                    if let Some(remote_report) = report.get("remote") {
                        println!("remote_checked: {}", remote_report["checked"]);
                        println!("remote_corrupt: {}", remote_report["corrupt"]);
                        if let Some(keys) = remote_report["corrupt_keys"].as_array() {
                            if !keys.is_empty() {
                                println!("remote_corrupt_keys: {}", remote_report["corrupt_keys"]);
                            }
                        }
                    }
                }
                if corrupt > 0 {
                    return Err(ExitCode::from(3));
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Adapters { command } => match command {
            AdaptersCommands::Ls => {
                let adapters = registered_adapters();
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.adapters.ls",
                        true,
                        json!(adapters),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                } else {
                    for a in adapters {
                        println!(
                            "{} {} effects={:?}",
                            a.adapter_id, a.adapter_version, a.effects
                        );
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
            AdaptersCommands::Dump => {
                let data = adapter_registry_dump();
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.adapters.dump",
                        true,
                        data,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&data).map_err(|_| ExitCode::from(3))?
                );
                Ok(ExitCode::SUCCESS)
            }
            AdaptersCommands::Doctor => {
                let docker = check_engine("docker");
                let podman = check_engine("podman");
                if cli.json {
                    let ok = docker
                        .get("status")
                        .and_then(|v| v.as_str())
                        .map(|v| v == "ok")
                        .unwrap_or(false)
                        || podman
                            .get("status")
                            .and_then(|v| v.as_str())
                            .map(|v| v == "ok")
                            .unwrap_or(false);
                    return emit_json(
                        &cli,
                        "dag.adapters.doctor",
                        ok,
                        json!({ "docker": docker, "podman": podman }),
                        Vec::new(),
                        if ok {
                            ExitCode::SUCCESS
                        } else {
                            ExitCode::from(3)
                        },
                    );
                } else {
                    println!("docker: {}", docker["status"]);
                    if let Some(v) = docker.get("version").and_then(|v| v.as_str()) {
                        println!("docker_version: {}", v);
                    }
                    println!("podman: {}", podman["status"]);
                    if let Some(v) = podman.get("version").and_then(|v| v.as_str()) {
                        println!("podman_version: {}", v);
                    }
                }
                if docker["status"] != "ok" && podman["status"] != "ok" {
                    return Err(ExitCode::from(3));
                }
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Export {
            run_dir,
            out,
            include_files,
        } => {
            let manifest = read_file(&run_dir.join("manifest.json"))?;
            let snapshot = read_file(&run_dir.join("graph.snapshot.json"))?;
            let nodes = read_node_traces(run_dir)?;
            let outputs = read_outputs_indexes(run_dir)?;
            let files = if *include_files {
                Some(collect_output_files(run_dir, &outputs)?)
            } else {
                None
            };
            let bundle = json!({
                "manifest": serde_json::from_str::<serde_json::Value>(&manifest).ok(),
                "graph_snapshot": serde_json::from_str::<serde_json::Value>(&snapshot).ok(),
                "node_traces": nodes,
                "outputs": outputs,
                "files": files,
            });
            fs::write(out, serde_json::to_vec_pretty(&bundle).unwrap())
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.export",
                    true,
                    json!({ "bundle": out }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else if !cli.quiet {
                println!("bundle: {}", out.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Import { file } => {
            let data = read_file(file)?;
            let val: serde_json::Value =
                serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
            let nodes = val
                .get("node_traces")
                .and_then(|v| v.as_object())
                .map(|o| o.len())
                .unwrap_or(0);
            let failed = val
                .get("node_traces")
                .and_then(|v| v.as_object())
                .map(|o| {
                    o.iter()
                        .filter_map(|(k, v)| {
                            if v.get("status")
                                == Some(&serde_json::Value::String("failed".to_string()))
                            {
                                Some(k.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let summary = json!({
                "has_manifest": val.get("manifest").is_some(),
                "has_graph_snapshot": val.get("graph_snapshot").is_some(),
                "nodes": nodes,
                "failed_nodes": failed,
            });
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.import",
                    true,
                    summary,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                println!("import summary: {}", summary);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Version => {
            let v = format!("bijux-dag {} ({})", env!("CARGO_PKG_VERSION"), SPEC_VERSION);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.version",
                    true,
                    json!({ "version": v }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", v);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn read_file(path: &Path) -> Result<String, ExitCode> {
    fs::read_to_string(path).map_err(|_| ExitCode::from(3))
}

fn parse_graph(input: &str) -> Result<Graph, ExitCode> {
    match parse_graph_strict(input) {
        Ok(g) => Ok(g),
        Err(GraphError::InvalidSpec(_)) => Err(ExitCode::from(2)),
        Err(_) => Err(ExitCode::from(3)),
    }
}

fn env_cache_dir() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

#[derive(serde::Deserialize)]
struct GraphSnapshot {
    graph: Graph,
    graph_fingerprint: String,
}

fn load_snapshot(run_dir: &Path) -> Result<GraphSnapshot, ExitCode> {
    let snap = read_file(&run_dir.join("graph.snapshot.json"))?;
    serde_json::from_str(&snap).map_err(|_| ExitCode::from(3))
}

fn read_node_traces(run_dir: &Path) -> Result<HashMap<String, serde_json::Value>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let trace_path = entry.path().join("trace.json");
            if trace_path.exists() {
                let data = read_file(&trace_path)?;
                let val: serde_json::Value =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                map.insert(node_id, val);
            }
        }
    }
    Ok(map)
}

fn read_outputs_indexes(run_dir: &Path) -> Result<HashMap<String, OutputsIndex>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let index_path = entry.path().join("outputs").join("index.json");
            if index_path.exists() {
                let data = read_file(&index_path)?;
                let val: OutputsIndex =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                map.insert(node_id, val);
            }
        }
    }
    Ok(map)
}

fn print_human_diff(diff: &serde_json::Value) {
    let manifest = diff["manifest"].as_object().map(|o| o.len()).unwrap_or(0);
    let graph_fp = diff["graph_fingerprint"].is_null();
    let nodes = diff["nodes"].as_object().map(|o| o.len()).unwrap_or(0);
    let outputs = diff["outputs"].as_object().map(|o| o.len()).unwrap_or(0);
    if manifest == 0 && graph_fp && nodes == 0 && outputs == 0 {
        println!("no differences");
        return;
    }
    println!("manifest changes: {}", manifest);
    println!("graph_fingerprint: {}", diff["graph_fingerprint"]);
    println!("nodes changed: {}", nodes);
    println!("outputs changed: {}", outputs);
}

fn collect_output_files(
    run_dir: &Path,
    outputs: &HashMap<String, OutputsIndex>,
) -> Result<serde_json::Value, ExitCode> {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut node_ids: Vec<String> = outputs.keys().cloned().collect();
    node_ids.sort();
    for node_id in node_ids {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        if let Some(index) = outputs.get(&node_id) {
            let mut entries = index.files.clone();
            entries.sort_by(|a, b| a.path.cmp(&b.path));
            for file in entries {
                let path = run_dir
                    .join("nodes")
                    .join(&node_id)
                    .join("outputs")
                    .join(&file.path);
                let bytes = fs::read(path).map_err(|_| ExitCode::from(3))?;
                let encoded = BASE64.encode(bytes);
                files.insert(file.path, encoded);
            }
        }
        out.insert(node_id, serde_json::to_value(files).unwrap());
    }
    Ok(serde_json::to_value(out).unwrap())
}

fn verify_cache_dir(dir: &Path) -> Result<serde_json::Value, ExitCode> {
    let mut checked = 0u64;
    let mut corrupt = 0u64;
    let mut corrupt_keys: Vec<String> = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(dir).map_err(|_| ExitCode::from(3))? {
            let entry = entry.map_err(|_| ExitCode::from(3))?;
            let path = entry.path();
            if path.is_dir() {
                checked += 1;
                let key = entry.file_name().to_string_lossy().to_string();
                let index_path = path.join("outputs").join("index.json");
                let meta_path = path.join("meta.json");
                if !index_path.exists() {
                    corrupt += 1;
                    corrupt_keys.push(key);
                    continue;
                }
                if !meta_path.exists() {
                    corrupt += 1;
                    corrupt_keys.push(key);
                    continue;
                }
                let data = fs::read_to_string(&index_path).map_err(|_| ExitCode::from(3))?;
                let index: OutputsIndex =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                for file in index.files {
                    let fpath = path.join("outputs").join(&file.path);
                    if !fpath.exists() {
                        corrupt += 1;
                        corrupt_keys.push(key.clone());
                        break;
                    }
                    let bytes = fs::read(&fpath).map_err(|_| ExitCode::from(3))?;
                    let sha = sha256_bytes(&bytes);
                    if sha != file.sha256 {
                        corrupt += 1;
                        corrupt_keys.push(key.clone());
                        break;
                    }
                }
            }
        }
    }
    corrupt_keys.sort();
    corrupt_keys.dedup();
    Ok(json!({ "checked": checked, "corrupt": corrupt, "corrupt_keys": corrupt_keys }))
}

fn verify_cache_dirs(
    local: &Path,
    remote: Option<&Path>,
) -> Result<serde_json::Value, ExitCode> {
    let local_report = verify_cache_dir(local)?;
    let mut checked_total = local_report["checked"].as_u64().unwrap_or(0);
    let mut corrupt_total = local_report["corrupt"].as_u64().unwrap_or(0);
    let mut out = json!({
        "local": local_report,
        "checked_total": checked_total,
        "corrupt_total": corrupt_total,
    });
    if let Some(remote_dir) = remote {
        let remote_report = verify_cache_dir(remote_dir)?;
        checked_total += remote_report["checked"].as_u64().unwrap_or(0);
        corrupt_total += remote_report["corrupt"].as_u64().unwrap_or(0);
        out["remote"] = remote_report;
        out["checked_total"] = json!(checked_total);
        out["corrupt_total"] = json!(corrupt_total);
    }
    Ok(out)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

fn pack_cache_entry(entry: &Path, out: &Path) -> Result<(), ExitCode> {
    let file = fs::File::create(out).map_err(|_| ExitCode::from(3))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(enc);
    builder
        .append_dir_all(".", entry)
        .map_err(|_| ExitCode::from(3))?;
    let enc = builder.into_inner().map_err(|_| ExitCode::from(3))?;
    enc.finish().map_err(|_| ExitCode::from(3))?;
    Ok(())
}

fn unpack_cache_entry(pack: &Path, cache_dir: &Path) -> Result<(), ExitCode> {
    let file = fs::File::open(pack).map_err(|_| ExitCode::from(3))?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    let tmp = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
    archive.unpack(tmp.path()).map_err(|_| ExitCode::from(3))?;
    let meta_path = tmp.path().join("meta.json");
    if !meta_path.exists() {
        return Err(ExitCode::from(3));
    }
    let mut meta: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let key = meta
        .get("node_fingerprint")
        .and_then(|v| v.as_str())
        .ok_or(ExitCode::from(3))?
        .to_string();
    let adapter_id = meta
        .get("adapter_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let adapter_version = meta
        .get("adapter_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !verify_cache_entry_cli(tmp.path(), &key, &adapter_id, &adapter_version)? {
        return Err(ExitCode::from(3));
    }
    if let Some(obj) = meta.as_object_mut() {
        obj.insert(
            "cache_source".to_string(),
            serde_json::Value::String("pack".to_string()),
        );
    }
    fs::write(&meta_path, serde_json::to_vec_pretty(&meta).unwrap())
        .map_err(|_| ExitCode::from(3))?;
    let dst = cache_dir.join(&key);
    if dst.exists() {
        let _ = fs::remove_dir_all(&dst);
    }
    copy_dir_all(tmp.path(), &dst).map_err(|_| ExitCode::from(3))?;
    Ok(())
}

fn verify_cache_entry_cli(
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
) -> Result<bool, ExitCode> {
    let index_path = entry.join("outputs").join("index.json");
    let meta_path = entry.join("meta.json");
    if !index_path.exists() || !meta_path.exists() {
        return Ok(false);
    }
    let meta: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(expected_key) {
        return Ok(false);
    }
    if !adapter_id.is_empty() && meta.get("adapter_id").and_then(|v| v.as_str()) != Some(adapter_id)
    {
        return Ok(false);
    }
    if !adapter_version.is_empty()
        && meta.get("adapter_version").and_then(|v| v.as_str()) != Some(adapter_version)
    {
        return Ok(false);
    }
    let data = fs::read_to_string(&index_path).map_err(|_| ExitCode::from(3))?;
    let index: OutputsIndex = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
    for file in index.files {
        let fpath = entry.join("outputs").join(&file.path);
        if !fpath.exists() {
            return Ok(false);
        }
        let bytes = fs::read(&fpath).map_err(|_| ExitCode::from(3))?;
        let sha = sha256_bytes(&bytes);
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    let mut entries: Vec<_> = fs::read_dir(src)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn check_engine(bin: &str) -> serde_json::Value {
    match std::process::Command::new(bin).arg("--version").output() {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            json!({"status":"ok","version":v})
        }
        _ => json!({"status":"missing"}),
    }
}

fn verify_run(run_dir: &Path, deep: bool) -> Result<serde_json::Value, ExitCode> {
    let mut errors = Vec::new();
    let manifest_path = run_dir.join("manifest.json");
    let manifest_data = fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?;
    let manifest: bijux_dag_artifacts::Manifest =
        serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
    let snapshot = load_snapshot(run_dir)?;
    let computed = snapshot.graph.graph_fingerprint().unwrap_or_default();
    if computed != snapshot.graph_fingerprint {
        errors.push(format!(
            "graph_fingerprint mismatch: {} != {}",
            computed, snapshot.graph_fingerprint
        ));
    }
    for node in &snapshot.graph.nodes {
        if manifest.policy.deny_network && node.effects.contains(&bijux_dag_core::Effect::Network) {
            errors.push(format!("policy deny_network violated by node {}", node.id));
        }
        if manifest.policy.deny_env && node.effects.contains(&bijux_dag_core::Effect::Env) {
            errors.push(format!("policy deny_env violated by node {}", node.id));
        }
        if manifest.policy.deny_clock && node.effects.contains(&bijux_dag_core::Effect::Clock) {
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
            let val: serde_json::Value =
                serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
            if deep {
                let typed_parse: Result<bijux_dag_artifacts::NodeTrace, _> = serde_json::from_str(&data);
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
        }
    }

    if deep {
        if serde_json::from_str::<bijux_dag_artifacts::Manifest>(&manifest_data).is_err() {
            errors.push("manifest schema parse failed".to_string());
        }
        if !outputs_index_path.exists() {
            errors.push("deep verify requires outputs/index.json".to_string());
        }
    }

    let status = if errors.is_empty() { "ok" } else { "error" };
    Ok(json!({
        "status": status,
        "mode": if deep { "deep" } else { "standard" },
        "artifacts_checked": {
            "manifest": manifest_path.exists(),
            "outputs_index": outputs_index_path.exists(),
            "outputs_files": outputs_count
        },
        "errors": errors
    }))
}

fn map_materialize_mode(arg: MaterializeModeArg) -> MaterializeMode {
    match arg {
        MaterializeModeArg::Copy => MaterializeMode::Copy,
        MaterializeModeArg::Hardlink => MaterializeMode::Hardlink,
        MaterializeModeArg::Symlink => MaterializeMode::Symlink,
    }
}


include!("graph_helpers.in.rs");
