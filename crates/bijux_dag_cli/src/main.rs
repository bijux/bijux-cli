mod diff;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_artifacts::OutputsIndex;
use bijux_dag_core::{parse_graph_strict, Graph, GraphError, Severity, SPEC_VERSION};
use bijux_dag_runtime::{registered_adapters, CacheMode, Runtime, RuntimeOptions};
use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "bijux-dag")]
#[command(about = "Bijux DAG CLI", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    quiet: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        dag: PathBuf,
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        print_fingerprints: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
    },
    Canonicalize {
        dag: PathBuf,
    },
    Fingerprint {
        dag: PathBuf,
    },
    Run {
        dag: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        latest: Option<PathBuf>,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        node_timeout_ms: Option<u64>,
        #[arg(long)]
        run_timeout_ms: Option<u64>,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
        #[arg(long)]
        hermetic: bool,
        #[arg(long)]
        only_tag: Option<String>,
        #[arg(long)]
        skip_tag: Option<String>,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Replay {
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        reuse_cache: bool,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
        #[arg(long)]
        hermetic: bool,
    },
    Diff {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long)]
        explain: bool,
    },
    Explain {
        run_dir: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        node: Option<String>,
    },
    Inspect {
        run_dir: PathBuf,
        #[arg(long)]
        node: String,
    },
    Status {
        run_dir: PathBuf,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    Adapters {
        #[command(subcommand)]
        command: AdaptersCommands,
    },
    Export {
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        include_files: bool,
    },
    Import {
        file: PathBuf,
    },
    Version,
}

#[derive(Subcommand)]
enum CacheCommands {
    Ls {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Gc {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Verify {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum AdaptersCommands {
    Ls,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CacheModeArg {
    Off,
    Read,
    Readwrite,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => code,
        Err(code) => code,
    }
}

fn run(cli: Cli) -> Result<ExitCode, ExitCode> {
    match cli.command {
        Commands::Validate {
            dag,
            strict,
            print_fingerprints,
            explain,
            deny_network,
            deny_env,
            deny_clock,
        } => {
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            let diags = graph.validate_with_warnings();
            let mut diags = diags;
            if deny_network || deny_env || deny_clock {
                for node in &graph.nodes {
                    if deny_network && node.effects.contains(&bijux_dag_core::Effect::Network) {
                        diags.push(bijux_dag_core::ValidationDiagnostic {
                            code: "E1013".to_string(),
                            message: "network effect denied by policy".to_string(),
                            path: format!("/nodes/{}/effects", node.id),
                            hint: Some("Remove network effect or drop --deny-network".to_string()),
                            severity: Severity::Error,
                        });
                    }
                    if deny_env && node.effects.contains(&bijux_dag_core::Effect::Env) {
                        diags.push(bijux_dag_core::ValidationDiagnostic {
                            code: "E1013".to_string(),
                            message: "env effect denied by policy".to_string(),
                            path: format!("/nodes/{}/effects", node.id),
                            hint: Some("Remove env effect or drop --deny-env".to_string()),
                            severity: Severity::Error,
                        });
                    }
                    if deny_clock && node.effects.contains(&bijux_dag_core::Effect::Clock) {
                        diags.push(bijux_dag_core::ValidationDiagnostic {
                            code: "E1013".to_string(),
                            message: "clock effect denied by policy".to_string(),
                            path: format!("/nodes/{}/effects", node.id),
                            hint: Some("Remove clock effect or drop --deny-clock".to_string()),
                            severity: Severity::Error,
                        });
                    }
                }
            }
            let has_errors = diags.iter().any(|d| d.severity == Severity::Error);
            let has_warnings = diags.iter().any(|d| d.severity == Severity::Warning);
            let fail = has_errors || (strict && has_warnings);

            if cli.json {
                let mut out = json!({"diagnostics": []});
                out["diagnostics"] = serde_json::to_value(&diags).unwrap();
                if print_fingerprints {
                    out["graph_fingerprint"] = json!(graph.graph_fingerprint().unwrap());
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
                    out["node_fingerprints"] = json!(nodes);
                }
                if explain {
                    let canonical = graph.canonicalize();
                    let order = canonical
                        .nodes
                        .iter()
                        .map(|n| n.id.clone())
                        .collect::<Vec<_>>();
                    out["canonical_order"] = json!(order);
                    out["resolved_params"] = json!(graph
                        .resolve_graph()
                        .map(|g| g.resolved_params)
                        .unwrap_or_default());
                    out["graph_fingerprint"] = json!(graph.graph_fingerprint().unwrap());
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
                    out["node_fingerprints"] = json!(nodes);
                }
                println!("{}", serde_json::to_string_pretty(&out).unwrap());
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
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            let json = graph.to_canonical_json().map_err(|_| ExitCode::from(3))?;
            println!("{}", json);
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fingerprint { dag } => {
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            let fp = graph.graph_fingerprint().map_err(|_| ExitCode::from(3))?;
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({"graph": fp})).unwrap()
                );
            } else {
                println!("{}", fp);
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
            hermetic,
        } => {
            let snapshot = load_snapshot(&run_dir)?;
            let runtime = Runtime::new();
            let cache_mode = match cache {
                CacheModeArg::Off => {
                    if reuse_cache {
                        CacheMode::Read
                    } else {
                        CacheMode::Off
                    }
                }
                CacheModeArg::Read => CacheMode::Read,
                CacheModeArg::Readwrite => CacheMode::ReadWrite,
            };
            let mut deny_network = deny_network;
            let mut deny_clock = deny_clock;
            if hermetic {
                deny_network = true;
                deny_clock = true;
            }
            let options = RuntimeOptions {
                jobs,
                cpu_budget,
                run_timeout_ms: None,
                node_timeout_ms: None,
                cache_mode,
                cache_dir: None,
                run_id,
                latest_symlink: None,
                policy: bijux_dag_runtime::Policy {
                    deny_network,
                    deny_env,
                    deny_clock,
                },
                only_tag: None,
                skip_tag: None,
            };
            let run_path = runtime
                .run(&snapshot.graph, out, options)
                .map_err(|_| ExitCode::from(3))?;
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
            let manifest_a = read_file(&run_a.join("manifest.json"))?;
            let manifest_b = read_file(&run_b.join("manifest.json"))?;
            let snap_a = load_snapshot(&run_a)?;
            let snap_b = load_snapshot(&run_b)?;
            let nodes_a = read_node_traces(&run_a)?;
            let nodes_b = read_node_traces(&run_b)?;
            let outputs_a = read_outputs_indexes(&run_a)?;
            let outputs_b = read_outputs_indexes(&run_b)?;
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
                println!("{}", serde_json::to_string_pretty(&diff).unwrap());
            } else {
                print_human_diff(&serde_json::to_value(&diff).unwrap());
                if explain {
                    println!("explain: graph fingerprint change implies cache invalidation");
                    println!("explain: node fingerprint changes indicate recomputation scope");
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
            hermetic,
            only_tag,
            skip_tag,
            cache,
            cache_dir,
        } => {
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            let runtime = Runtime::new();
            let mut deny_network = deny_network;
            let mut deny_clock = deny_clock;
            if hermetic {
                deny_network = true;
                deny_clock = true;
            }
            let options = RuntimeOptions {
                jobs,
                cpu_budget,
                run_timeout_ms,
                node_timeout_ms,
                cache_mode: match cache {
                    CacheModeArg::Off => CacheMode::Off,
                    CacheModeArg::Read => CacheMode::Read,
                    CacheModeArg::Readwrite => CacheMode::ReadWrite,
                },
                cache_dir,
                run_id,
                latest_symlink: latest,
                policy: bijux_dag_runtime::Policy {
                    deny_network,
                    deny_env,
                    deny_clock,
                },
                only_tag,
                skip_tag,
            };
            let run_path = runtime
                .run(&graph, out, options)
                .map_err(|_| ExitCode::from(3))?;
            if !cli.quiet {
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({"run_dir": run_path})).unwrap()
                    );
                } else {
                    println!("run dir: {}", run_path.display());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Explain {
            run_dir,
            json,
            node,
        } => {
            let manifest_path = run_dir.join("manifest.json");
            let manifest = read_file(&manifest_path)?;
            if let Some(node_id) = node {
                let snapshot = load_snapshot(&run_dir)?;
                let trace = read_file(&run_dir.join("nodes").join(&node_id).join("trace.json"))?;
                let node_info = snapshot
                    .graph
                    .nodes
                    .iter()
                    .find(|n| n.id == node_id)
                    .ok_or(ExitCode::from(3))?;
                let deps = snapshot
                    .graph
                    .edges
                    .iter()
                    .filter(|e| e.to.node_id == node_id)
                    .map(|e| e.from.node_id.clone())
                    .collect::<Vec<_>>();
                let outputs_index = read_file(
                    &run_dir
                        .join("nodes")
                        .join(&node_id)
                        .join("outputs")
                        .join("index.json"),
                )
                .ok();
                let resolved_params = read_file(
                    &run_dir
                        .join("nodes")
                        .join(&node_id)
                        .join("resolved_params.json"),
                )
                .ok();
                let outputs = node_info.outputs.clone();
                let inputs = node_info.inputs.clone();
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
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
                        }))
                        .unwrap()
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
            } else if json {
                let m: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
                let status = m.get("status").cloned().unwrap_or_default();
                let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
                let counts = m.get("node_counts").cloned().unwrap_or_default();
                let nodes = read_node_traces(&run_dir).unwrap_or_default();
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "status": status,
                        "graph_fingerprint": graph_fp,
                        "node_counts": counts,
                        "failed_nodes": failed,
                    }))
                    .unwrap()
                );
            } else {
                let m: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
                let status = m.get("status").cloned().unwrap_or_default();
                let graph_fp = m.get("graph_fingerprint").cloned().unwrap_or_default();
                let counts = m.get("node_counts").cloned().unwrap_or_default();
                let nodes = read_node_traces(&run_dir).unwrap_or_default();
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
        Commands::Inspect { run_dir, node } => {
            let trace = read_file(&run_dir.join("nodes").join(&node).join("trace.json"))?;
            let index = read_file(
                &run_dir
                    .join("nodes")
                    .join(&node)
                    .join("outputs")
                    .join("index.json"),
            )?;
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &json!({"manifest": manifest, "traces": statuses})
                    )
                    .unwrap()
                );
            } else {
                println!("manifest:\n{}", manifest);
                println!("traces: {}", statuses.len());
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Cache { command } => match command {
            CacheCommands::Ls { cache_dir } => {
                let dir = cache_dir.or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                if dir.exists() {
                    for entry in fs::read_dir(dir).map_err(|_| ExitCode::from(3))? {
                        let entry = entry.map_err(|_| ExitCode::from(3))?;
                        println!("{}", entry.file_name().to_string_lossy());
                    }
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Gc { cache_dir } => {
                let _dir = cache_dir.or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                println!("cache gc stub");
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Verify { cache_dir } => {
                let dir = cache_dir.or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                let report = verify_cache_dir(&dir)?;
                let corrupt = report["corrupt"].as_u64().unwrap_or(0);
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                } else {
                    println!("checked: {}", report["checked"]);
                    println!("corrupt: {}", report["corrupt"]);
                    if let Some(keys) = report["corrupt_keys"].as_array() {
                        if !keys.is_empty() {
                            println!("corrupt_keys: {}", report["corrupt_keys"]);
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
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!(adapters)).unwrap()
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
        },
        Commands::Export {
            run_dir,
            out,
            include_files,
        } => {
            let manifest = read_file(&run_dir.join("manifest.json"))?;
            let snapshot = read_file(&run_dir.join("graph.snapshot.json"))?;
            let nodes = read_node_traces(&run_dir)?;
            let outputs = read_outputs_indexes(&run_dir)?;
            let files = if include_files {
                Some(collect_output_files(&run_dir, &outputs)?)
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
            fs::write(&out, serde_json::to_vec_pretty(&bundle).unwrap())
                .map_err(|_| ExitCode::from(3))?;
            if !cli.quiet {
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({"bundle": out})).unwrap()
                    );
                } else {
                    println!("bundle: {}", out.display());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Import { file } => {
            let data = read_file(&file)?;
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
                println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            } else {
                println!("import summary: {}", summary);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Version => {
            println!("bijux-dag {} ({})", env!("CARGO_PKG_VERSION"), SPEC_VERSION);
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn read_file(path: &PathBuf) -> Result<String, ExitCode> {
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

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}
