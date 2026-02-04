mod diff;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_artifacts::{OutputsIndex, RunOutputsIndex};
use bijux_dag_core::{parse_graph_strict, Graph, GraphError, Severity, SPEC_VERSION};
use bijux_dag_runtime::{
    registered_adapters, CacheMode, MaterializeMode, Runtime, RuntimeOptions, Selector, SelectorSet,
};
use clap::{Parser, Subcommand, ValueEnum};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tar::{Archive, Builder};

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
    Init {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
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
    Lint {
        dag: PathBuf,
        #[arg(long)]
        strict: bool,
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
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        exclude: Vec<String>,
        #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
        materialize_inputs: MaterializeModeArg,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        remote_cache_dir: Option<PathBuf>,
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
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        exclude: Vec<String>,
        #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
        materialize_inputs: MaterializeModeArg,
        #[arg(long)]
        remote_cache_dir: Option<PathBuf>,
    },
    Graph {
        dag: PathBuf,
        #[arg(long, value_enum, default_value_t = GraphFormatArg::Dot)]
        format: GraphFormatArg,
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
    VerifyRun {
        run_dir: PathBuf,
    },
    Doctor,
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    Compat,
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
    Pack {
        node_fp: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Unpack {
        pack: PathBuf,
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
    Doctor,
}

#[derive(Subcommand)]
enum MigrateCommands {
    Dag {
        file: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
    Run {
        run_dir: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum CacheModeArg {
    Off,
    Read,
    Readwrite,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum MaterializeModeArg {
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum GraphFormatArg {
    Dot,
}

#[derive(Debug, Serialize)]
struct LintDiagnostic {
    code: String,
    message: String,
    path: String,
    hint: Option<String>,
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
        Commands::Init { dir } => {
            let base = dir.unwrap_or_else(|| PathBuf::from("."));
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
                  "params": {"argv": ["/bin/sh","-c","cat ../inputs/const1/in/out > ../outputs/out"]},
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
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "dag": dag_path,
                        "runs": runs_dir
                    }))
                    .unwrap()
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
        Commands::Lint { dag, strict } => {
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            let lint = lint_graph(&graph);
            let has_warnings = !lint.is_empty();
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({ "warnings": lint })).unwrap()
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
        Commands::Graph { dag, format } => {
            let input = read_file(&dag)?;
            let graph = parse_graph(&input)?;
            match format {
                GraphFormatArg::Dot => {
                    println!("{}", graph_to_dot(&graph));
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
            hermetic,
            select,
            exclude,
            materialize_inputs,
            remote_cache_dir,
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
            let selectors = parse_selectors(&select, &exclude)?;
            let options = RuntimeOptions {
                jobs,
                cpu_budget,
                run_timeout_ms: None,
                node_timeout_ms: None,
                materialize_inputs: map_materialize_mode(materialize_inputs),
                cache_mode,
                cache_dir: None,
                remote_cache_dir,
                run_id,
                latest_symlink: None,
                policy: bijux_dag_runtime::Policy {
                    deny_network,
                    deny_env,
                    deny_clock,
                },
                selectors,
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
            select,
            exclude,
            materialize_inputs,
            cache,
            cache_dir,
            remote_cache_dir,
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
            let selectors = parse_selectors(&select, &exclude)?;
            let options = RuntimeOptions {
                jobs,
                cpu_budget,
                run_timeout_ms,
                node_timeout_ms,
                materialize_inputs: map_materialize_mode(materialize_inputs),
                cache_mode: match cache {
                    CacheModeArg::Off => CacheMode::Off,
                    CacheModeArg::Read => CacheMode::Read,
                    CacheModeArg::Readwrite => CacheMode::ReadWrite,
                },
                cache_dir,
                remote_cache_dir,
                run_id,
                latest_symlink: latest,
                policy: bijux_dag_runtime::Policy {
                    deny_network,
                    deny_env,
                    deny_clock,
                },
                selectors,
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
        Commands::VerifyRun { run_dir } => {
            let report = verify_run(&run_dir)?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("status: {}", report["status"]);
            }
            if report["status"] != "ok" {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Doctor => {
            let report = doctor_report()?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("status: {}", report["status"]);
            }
            if report["status"] != "ok" {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Migrate { command } => {
            let msg = match command {
                MigrateCommands::Dag { file, from, to } => migrate_dag(&file, &from, &to)?,
                MigrateCommands::Run { run_dir, from, to } => migrate_run(&run_dir, &from, &to)?,
            };
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({ "status": "ok", "message": msg }))
                        .unwrap()
                );
            } else if !cli.quiet {
                println!("{}", msg);
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Compat => {
            let report = run_compat_suite()?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("status: {}", report["status"]);
            }
            if report["status"] != "ok" {
                return Err(ExitCode::from(3));
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
            CacheCommands::Pack {
                node_fp,
                out,
                cache_dir,
            } => {
                let dir = cache_dir.or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                let entry = dir.join(&node_fp);
                if !entry.exists() {
                    return Err(ExitCode::from(3));
                }
                pack_cache_entry(&entry, &out)?;
                if !cli.quiet {
                    println!("pack: {}", out.display());
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Unpack { pack, cache_dir } => {
                let dir = cache_dir.or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                unpack_cache_entry(&pack, &dir)?;
                if !cli.quiet {
                    println!("unpacked: {}", pack.display());
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
            AdaptersCommands::Doctor => {
                let docker = check_engine("docker");
                let podman = check_engine("podman");
                if cli.json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&json!({
                            "docker": docker,
                            "podman": podman
                        }))
                        .unwrap()
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
    let meta: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let key = meta
        .get("node_fingerprint")
        .and_then(|v| v.as_str())
        .ok_or(ExitCode::from(3))?;
    let adapter_id = meta
        .get("adapter_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let adapter_version = meta
        .get("adapter_version")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !verify_cache_entry_cli(tmp.path(), key, adapter_id, adapter_version)? {
        return Err(ExitCode::from(3));
    }
    let dst = cache_dir.join(key);
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

fn verify_run(run_dir: &Path) -> Result<serde_json::Value, ExitCode> {
    let mut errors = Vec::new();
    let snapshot = load_snapshot(run_dir)?;
    let computed = snapshot.graph.graph_fingerprint().unwrap_or_default();
    if computed != snapshot.graph_fingerprint {
        errors.push(format!(
            "graph_fingerprint mismatch: {} != {}",
            computed, snapshot.graph_fingerprint
        ));
    }

    let outputs_index_path = run_dir.join("outputs").join("index.json");
    if !outputs_index_path.exists() {
        errors.push("missing outputs/index.json".to_string());
    } else {
        let data = fs::read_to_string(&outputs_index_path).map_err(|_| ExitCode::from(3))?;
        let index: RunOutputsIndex = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
        for file in index.files {
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

    let status = if errors.is_empty() { "ok" } else { "error" };
    Ok(json!({ "status": status, "errors": errors }))
}

fn map_materialize_mode(arg: MaterializeModeArg) -> MaterializeMode {
    match arg {
        MaterializeModeArg::Copy => MaterializeMode::Copy,
        MaterializeModeArg::Hardlink => MaterializeMode::Hardlink,
        MaterializeModeArg::Symlink => MaterializeMode::Symlink,
    }
}

fn parse_selectors(include: &[String], exclude: &[String]) -> Result<SelectorSet, ExitCode> {
    let mut set = SelectorSet {
        include: Vec::new(),
        exclude: Vec::new(),
    };
    for raw in include {
        set.include.push(parse_selector(raw)?);
    }
    for raw in exclude {
        set.exclude.push(parse_selector(raw)?);
    }
    Ok(set)
}

fn parse_selector(raw: &str) -> Result<Selector, ExitCode> {
    if let Some(rest) = raw.strip_prefix("id:") {
        return Ok(Selector::IdPrefix(rest.to_string()));
    }
    if let Some(rest) = raw.strip_prefix("tag:") {
        return Ok(Selector::Tag(rest.to_string()));
    }
    if let Some(rest) = raw.strip_prefix("kind:") {
        return Ok(Selector::Kind(rest.to_string()));
    }
    Err(ExitCode::from(2))
}

fn lint_graph(graph: &Graph) -> Vec<LintDiagnostic> {
    let mut out = Vec::new();
    for diag in graph.validate_with_warnings() {
        if diag.severity == Severity::Warning {
            out.push(LintDiagnostic {
                code: diag.code,
                message: diag.message,
                path: diag.path,
                hint: diag.hint,
            });
        }
    }
    let mut used_outputs: std::collections::BTreeSet<(String, String)> =
        std::collections::BTreeSet::new();
    for edge in &graph.edges {
        used_outputs.insert((edge.from.node_id.clone(), edge.from.port.clone()));
    }
    for node in &graph.nodes {
        for outp in &node.outputs {
            if !used_outputs.contains(&(node.id.clone(), outp.name.clone())) {
                out.push(LintDiagnostic {
                    code: "L1001".to_string(),
                    message: format!("unused output: {}", outp.name),
                    path: format!("/nodes/{}/outputs", node.id),
                    hint: Some("Remove or connect this output".to_string()),
                });
            }
        }
        if node.resources.is_none() {
            out.push(LintDiagnostic {
                code: "L1002".to_string(),
                message: "missing resource hints".to_string(),
                path: format!("/nodes/{}/resources", node.id),
                hint: Some("Set resources.cpu/mem_mb for scheduling".to_string()),
            });
        }
        if node.effects.iter().any(|e| {
            matches!(
                e,
                bijux_dag_core::Effect::Network
                    | bijux_dag_core::Effect::Env
                    | bijux_dag_core::Effect::Clock
            )
        }) {
            out.push(LintDiagnostic {
                code: "L1003".to_string(),
                message: "broad effects declared".to_string(),
                path: format!("/nodes/{}/effects", node.id),
                hint: Some("Use minimal effects required".to_string()),
            });
        }
    }
    out
}

fn graph_to_dot(graph: &Graph) -> String {
    let g = graph.canonicalize();
    let mut out = String::from("digraph bijux {\n");
    for node in &g.nodes {
        out.push_str(&format!("  \"{}\";\n", node.id));
    }
    for edge in &g.edges {
        out.push_str(&format!(
            "  \"{}\" -> \"{}\" [label=\"{}->{}\"];\n",
            edge.from.node_id, edge.to.node_id, edge.from.port, edge.to.port
        ));
    }
    out.push_str("}\n");
    out
}

fn doctor_report() -> Result<serde_json::Value, ExitCode> {
    let cache_dir = env_cache_dir();
    let cache_status = if let Some(dir) = cache_dir.as_ref() {
        if fs::create_dir_all(dir).is_ok() {
            let test = dir.join(".__bijux_write_test");
            let writable = fs::write(&test, b"ok").is_ok();
            let _ = fs::remove_file(&test);
            if writable {
                json!({"status":"ok","path":dir})
            } else {
                json!({"status":"error","path":dir})
            }
        } else {
            json!({"status":"error","path":dir})
        }
    } else {
        json!({"status":"missing"})
    };

    let docker = check_engine("docker");
    let podman = check_engine("podman");
    let adapters = registered_adapters();

    let hardlink_ok = {
        let dir = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        let _ = fs::write(&a, b"ok");
        fs::hard_link(&a, &b).is_ok()
    };

    let status = if cache_status["status"] == "error" {
        "error"
    } else {
        "ok"
    };

    Ok(json!({
        "status": status,
        "cache": cache_status,
        "container": { "docker": docker, "podman": podman },
        "adapters": adapters,
        "filesystem": { "hardlink": hardlink_ok },
        "policy": { "clock": "allowed_by_default" }
    }))
}

fn migrate_dag(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let input = read_file(path)?;
    let graph = parse_graph(&input)?;
    if graph.spec != from {
        return Err(ExitCode::from(3));
    }
    if from == to {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

fn migrate_run(path: &Path, from: &str, to: &str) -> Result<String, ExitCode> {
    let snapshot = load_snapshot(path)?;
    if snapshot.graph.spec != from {
        return Err(ExitCode::from(3));
    }
    if from == to {
        return Ok("no migration needed".to_string());
    }
    Err(ExitCode::from(3))
}

fn run_compat_suite() -> Result<serde_json::Value, ExitCode> {
    let base = PathBuf::from("tests/compat/v0.1");
    if !base.exists() {
        return Ok(json!({"status":"ok","errors":[]}));
    }
    let mut errors = Vec::new();
    let mut entries: Vec<_> = fs::read_dir(&base)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with(".dag.json"))
            .unwrap_or(false)
        {
            continue;
        }
        let stem = path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .replace(".dag.json", "");
        let canonical_path = base.join(format!("{}.canonical.json", stem));
        let graph_fp_path = base.join(format!("{}.graph_fingerprint", stem));
        let node_fp_path = base.join(format!("{}.node_fingerprints.json", stem));

        let input = read_file(&path)?;
        let graph = parse_graph(&input)?;
        let canonical = graph.to_canonical_json().map_err(|_| ExitCode::from(3))?;
        let expected = read_file(&canonical_path).unwrap_or_default();
        if canonical.trim() != expected.trim() {
            errors.push(format!("canonical mismatch: {}", stem));
        }
        let fp = graph.graph_fingerprint().map_err(|_| ExitCode::from(3))?;
        let expected_fp = read_file(&graph_fp_path)
            .unwrap_or_default()
            .trim()
            .to_string();
        if fp != expected_fp {
            errors.push(format!("graph fingerprint mismatch: {}", stem));
        }
        let resolved = graph.resolve_graph().ok().map(|g| g.resolved_params);
        let mut nodes = serde_json::Map::new();
        for n in &graph.nodes {
            let fp = resolved
                .as_ref()
                .and_then(|m| m.get(&n.id))
                .and_then(|p| graph.node_fingerprint_with_params(n, p).ok())
                .unwrap_or_else(|| graph.node_fingerprint(n).unwrap());
            nodes.insert(n.id.clone(), json!(fp));
        }
        let expected_nodes = read_file(&node_fp_path).unwrap_or_default();
        let expected_val: serde_json::Value =
            serde_json::from_str(&expected_nodes).unwrap_or_else(|_| json!({}));
        if json!(nodes) != expected_val {
            errors.push(format!("node fingerprint mismatch: {}", stem));
        }
    }
    let status = if errors.is_empty() { "ok" } else { "error" };
    Ok(json!({ "status": status, "errors": errors }))
}
