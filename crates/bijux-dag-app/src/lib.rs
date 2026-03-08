#![allow(dead_code)]

mod cache;
#[path = "cache/cmd.rs"]
mod cache_cmd;
mod capability_matrix;
#[path = "commands/cli_model.rs"]
mod cli_model;
mod commands;
#[path = "commands/config_resolution.rs"]
mod config_resolution;
#[path = "commands/config_surface.rs"]
mod config_surface;
#[path = "replay/diff.rs"]
mod diff;
#[path = "inspect/doctor_cmd.rs"]
mod doctor_cmd;
mod explain;
#[path = "explain/cmd.rs"]
mod explain_cmd;
#[path = "commands/export_cmd.rs"]
mod export_cmd;
mod format;
#[path = "read/fs_input.rs"]
mod fs_input;
mod graph;
#[path = "graph/cmd.rs"]
mod graph_cmd;
#[path = "graph/helpers.rs"]
mod graph_helpers;
#[path = "commands/import_cmd.rs"]
mod import_cmd;
mod inspect;
#[path = "inspect/service.rs"]
mod inspect_service;
mod migrate;
mod read;
#[path = "read/read_graph.rs"]
mod read_graph;
mod replay;
#[path = "replay/cmd.rs"]
mod replay_cmd;
#[path = "replay/service.rs"]
mod replay_service;
mod routes;
#[path = "commands/run_cmd.rs"]
mod run_cmd;
#[path = "inspect/run_views.rs"]
mod run_views;
#[path = "inspect/status_cmd.rs"]
mod status_cmd;
#[path = "graph/validate_cmd.rs"]
mod validate_cmd;
mod write;

pub use config_surface::{
    config_fingerprint, default_runtime_config, normalize_runtime_config, policy_evaluation_trace,
    resolve_effective_config, CacheModeSurface, MaterializeInputsSurface,
    PartialRuntimeSurfaceConfig, PolicySurfaceConfig, RuntimeSurfaceConfig,
};
pub use run_views::{
    doctor_run, explain_failure, explain_run_id, format_inspect_human, format_show_human,
    inspect_summary, list_runs, resolve_run_dir, run_timeline, run_tree, runs_compare,
    runs_failures, runs_flakes, runs_history, runs_history_query, runs_summary, runs_trend,
};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_artifacts::{OutputsIndex, RunOutputsIndex};
use bijux_dag_core::{
    lower_graph_to_execution_plan, parse_graph_strict, planner_diagnostics_from_error, Graph,
    GraphError, PlanOptions, Severity, SPEC_VERSION,
};
use bijux_dag_runtime::{
    adapter_registry_dump, build_plan, registered_adapters, CacheMode, MaterializeMode, Runtime,
    RuntimeConfig,
};
#[cfg(test)]
use bijux_dag_testkit as _;
use capability_matrix::backend_capability_payload;
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use commands::{
    AdaptersCommands, CacheCommands, Commands, ConfigCommands, DagCli, GraphFormatArg,
    HashCommands, MaterializeModeArg, MigrateCommands, PolicyCommands,
};
use config_resolution::{
    show_effective_config, show_effective_policy, ShowEffectiveConfigRequest,
    ShowEffectivePolicyRequest,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use graph_helpers::*;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tar::{Archive, Builder};
// Keep the dependency reachable at the crate root for strict target dependency checks.
use thiserror as _;

pub fn dag_command() -> clap::Command {
    DagCli::command().name("dag").subcommand_required(false)
}

pub fn dag_run(matches: &ArgMatches) -> Result<ExitCode, ExitCode> {
    if matches.subcommand_name().is_none() {
        let mut cmd = dag_command();
        let _ = cmd.print_help();
        println!();
        return Ok(ExitCode::SUCCESS);
    }
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
    status: String,
    command: String,
    data: Value,
    diagnostics: Vec<Value>,
    error: Option<JsonError>,
}

#[derive(Debug, Serialize)]
struct JsonError {
    category: String,
    code: String,
    message: String,
    exit_code: u8,
}

pub(crate) fn emit_json(
    cli: &DagCli,
    command: &str,
    ok: bool,
    data: Value,
    diagnostics: Vec<Value>,
    code: ExitCode,
) -> Result<ExitCode, ExitCode> {
    if !cli.quiet {
        let exit_value = exit_code_to_u8(code);
        let error = if ok {
            None
        } else {
            let (category, stable_code) = classify_exit(command, exit_value);
            Some(JsonError {
                category: category.to_string(),
                code: stable_code.to_string(),
                message: "command execution failed".to_string(),
                exit_code: exit_value,
            })
        };
        let envelope = JsonEnvelope {
            ok,
            status: if ok { "ok" } else { "invalid" }.to_string(),
            command: command.to_string(),
            data,
            diagnostics,
            error,
        };
        println!("{}", serde_json::to_string_pretty(&envelope).unwrap());
    }
    if ok {
        Ok(code)
    } else {
        Err(code)
    }
}

fn exit_code_to_u8(code: ExitCode) -> u8 {
    if code == ExitCode::SUCCESS {
        0
    } else if code == ExitCode::from(2) {
        2
    } else if code == ExitCode::from(3) {
        3
    } else {
        1
    }
}

fn classify_exit(command: &str, code: u8) -> (&'static str, &'static str) {
    match (command, code) {
        (_, 2) if command.contains("validate") || command.contains("lint") => {
            ("validation", "BJX-VALIDATION-001")
        }
        (_, 2) if command.contains("replay") => ("replay", "BJX-REPLAY-001"),
        (_, 2) if command.contains("cache") => ("cache", "BJX-CACHE-001"),
        (_, 2) => ("compatibility", "BJX-COMPAT-001"),
        (_, 3) => ("io", "BJX-IO-001"),
        _ => ("internal", "BJX-INTERNAL-001"),
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
        } => routes::validate_routes::handle_validate_command(
            &cli,
            dag,
            *strict,
            *print_fingerprints,
            *explain,
        ),
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
        Commands::GraphLint { dag, strict } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let lint = lint_graph(&graph);
            let has_warnings = !lint.is_empty();
            if cli.json {
                let diagnostics: Vec<Value> = lint
                    .iter()
                    .map(|d| serde_json::to_value(d).unwrap())
                    .collect();
                let code = if *strict && has_warnings {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                };
                return emit_json(
                    &cli,
                    "dag.graph-lint",
                    !(*strict && has_warnings),
                    json!({}),
                    diagnostics,
                    code,
                );
            }
            for warn in &lint {
                println!("WARN {} {} {}", warn.code, warn.path, warn.message);
            }
            if *strict && has_warnings {
                return Err(ExitCode::from(2));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fingerprint { dag, explain } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let explained = graph
                .graph_fingerprint_explain()
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.fingerprint",
                    true,
                    if *explain {
                        serde_json::to_value(&explained).map_err(|_| ExitCode::from(3))?
                    } else {
                        json!({"graph": explained.graph_id.as_str()})
                    },
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            } else {
                if *explain {
                    println!("{}", explained.graph_id.as_str());
                    println!("hash_algorithm={}", explained.hash_algorithm);
                    println!(
                        "canonical_json_bytes_len={}",
                        explained.canonical_json_bytes_len
                    );
                } else {
                    println!("{}", explained.graph_id.as_str());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Hash { command } => match command {
            HashCommands::Graph { dag, explain } => {
                let input = read_file(dag)?;
                let graph = parse_graph(&input)?;
                let explained = graph
                    .graph_fingerprint_explain()
                    .map_err(|_| ExitCode::from(3))?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.hash.graph",
                        true,
                        if *explain {
                            serde_json::to_value(&explained).map_err(|_| ExitCode::from(3))?
                        } else {
                            json!({"graph_id": explained.graph_id.as_str()})
                        },
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", explained.graph_id.as_str());
                if *explain {
                    println!("hash_algorithm={}", explained.hash_algorithm);
                    println!(
                        "canonical_json_bytes_len={}",
                        explained.canonical_json_bytes_len
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
            HashCommands::Run { run_dir } => {
                let digest = hash_run_dir(run_dir)?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.hash.run",
                        true,
                        json!({"run_hash": digest}),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{digest}");
                Ok(ExitCode::SUCCESS)
            }
            HashCommands::Artifact { file } => {
                let bytes = fs::read(file).map_err(|_| ExitCode::from(3))?;
                let sha256 = bijux_dag_artifacts::hash::sha256_hex(&bytes);
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.hash.artifact",
                        true,
                        json!({
                            "artifact_sha256": sha256,
                            "bytes_len": bytes.len()
                        }),
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{sha256}");
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::ArtifactInspect {
            run_dir,
            artifact_id,
        } => {
            let details = inspect_artifact(run_dir, artifact_id)?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.artifact-inspect",
                    true,
                    details,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&details).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::CanonicalBytes { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let bytes = graph
                .canonical_json_bytes()
                .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.canonical-bytes",
                    true,
                    json!({
                        "bytes_len": bytes.len(),
                        "utf8": String::from_utf8_lossy(&bytes),
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", String::from_utf8_lossy(&bytes));
            Ok(ExitCode::SUCCESS)
        }
        Commands::CanonicalDiff { dag } => {
            let input = read_file(dag)?;
            let raw: Value = serde_json::from_str(&input).map_err(|_| ExitCode::from(2))?;
            let graph = parse_graph(&input)?;
            let canonical: Value =
                serde_json::from_str(&graph.to_canonical_json().map_err(|_| ExitCode::from(3))?)
                    .map_err(|_| ExitCode::from(3))?;
            let mut changed_paths = Vec::new();
            collect_json_diff_paths("", &raw, &canonical, &mut changed_paths);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.canonical-diff",
                    true,
                    json!({
                        "changed_paths": changed_paths,
                        "raw": raw,
                        "canonical": canonical
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for p in changed_paths {
                println!("{p}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::ShowEffectiveGraph { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let canonical = graph.canonicalize();
            let payload = serde_json::to_value(&canonical).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.show-effective-graph",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::ShowEffectivePlan { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let plan = build_plan(&graph, &RuntimeConfig::default());
            let payload = serde_json::to_value(&plan).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.show-effective-plan",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::Plan { command } => routes::plan_routes::handle_plan_command(&cli, command),
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
            dry_run,
            prove,
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
        } => routes::replay_routes::handle_replay_command(
            &cli,
            run_dir,
            out,
            *dry_run,
            *prove,
            *reuse_cache,
            *cache,
            *jobs,
            run_id.clone(),
            *cpu_budget,
            *deny_network,
            *deny_env,
            *deny_clock,
            *clean_env,
            *hermetic,
            select,
            exclude,
            *materialize_inputs,
            remote_cache_dir.clone(),
        ),
        Commands::Prove { run_dir } => {
            routes::prove_verify_routes::handle_prove_command(&cli, run_dir)
        }
        Commands::ProofSummary { run_dir } => {
            routes::prove_verify_routes::handle_proof_summary_command(&cli, run_dir)
        }
        Commands::Runs { command } => routes::runs_routes::handle_runs_command(&cli, command),
        Commands::Diff {
            run_a,
            run_b,
            mode: _mode,
            explain,
        } => routes::diff_routes::handle_diff_command(&cli, run_a, run_b, *explain, "dag.diff"),
        Commands::WhyRerun { run_a, run_b } => {
            let payload = routes::diagnostics_routes::why_rerun_payload(run_a, run_b)?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.why-rerun",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::WhyCacheMissed {
            key,
            expected_adapter_id,
            expected_adapter_version,
            cache_dir,
        } => {
            let dir = cache_dir
                .clone()
                .or_else(env_cache_dir)
                .unwrap_or_else(|| PathBuf::from(".bijux/cache"));
            let report =
                explain_cache_key(&dir, key, expected_adapter_id, expected_adapter_version)?;
            let payload = json!({
                "cache_dir": dir,
                "key": key,
                "eligible": report["eligible"],
                "reasons": report["reasons"],
                "meta": report["meta"]
            });
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.why-cache-missed",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::TraceArtifact {
            run_dir,
            artifact_id,
        } => {
            let payload = routes::diagnostics_routes::trace_artifact_payload(run_dir, artifact_id)?;
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.trace-artifact",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
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
        } => routes::run_routes::handle_run_command(
            &cli,
            routes::run_routes::RunRouteRequest {
                dag,
                out,
                run_id: run_id.clone(),
                latest: latest.clone(),
                jobs: *jobs,
                cpu_budget: *cpu_budget,
                node_timeout_ms: *node_timeout_ms,
                run_timeout_ms: *run_timeout_ms,
                deny_network: *deny_network,
                deny_env: *deny_env,
                deny_clock: *deny_clock,
                clean_env: *clean_env,
                hermetic: *hermetic,
                select,
                exclude,
                materialize_inputs: *materialize_inputs,
                cache: *cache,
                cache_dir: cache_dir.clone(),
                remote_cache_dir: remote_cache_dir.clone(),
            },
        ),
        Commands::Explain { run_dir, node } => {
            routes::inspect_routes::handle_explain_command(&cli, run_dir, node)
        }
        Commands::Node { run_dir, id: node } => {
            routes::inspect_routes::handle_node_command(&cli, run_dir, node)
        }
        Commands::Status { run_dir } => routes::inspect_routes::handle_status_command(&cli, run_dir),
        Commands::Verify {
            run_dir,
            deep,
            strict,
        } => routes::prove_verify_routes::handle_verify_command(&cli, run_dir, *deep, *strict),
        Commands::Fsck { run_dir, strict } => {
            routes::prove_verify_routes::handle_fsck_command(&cli, run_dir, *strict)
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
                MigrateCommands::Dag {
                    file,
                    from,
                    to,
                    dry_run,
                } => {
                    let result = migrate_dag(file, from, to)?;
                    if *dry_run {
                        format!("dry-run: {result}")
                    } else {
                        result
                    }
                }
                MigrateCommands::Run {
                    run_dir,
                    from,
                    to,
                    dry_run,
                } => {
                    let result = migrate_run(run_dir, from, to)?;
                    if *dry_run {
                        format!("dry-run: {result}")
                    } else {
                        result
                    }
                }
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
            CacheCommands::Explain {
                cache_dir,
                key,
                expected_adapter_id,
                expected_adapter_version,
            } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let report = explain_cache_key(
                    &dir,
                    key,
                    expected_adapter_id.as_deref().unwrap_or(""),
                    expected_adapter_version.as_deref().unwrap_or(""),
                )?;
                let hit = report["eligible"].as_bool().unwrap_or(false);
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.explain",
                        true,
                        report,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                if !hit {
                    return Err(ExitCode::from(3));
                }
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Stats { cache_dir } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let report = cache_stats(&dir)?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.stats",
                        true,
                        report,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::PruneSimulate { cache_dir } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let report = cache_prune_simulate(&dir)?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.prune-simulate",
                        true,
                        report,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                Ok(ExitCode::SUCCESS)
            }
            CacheCommands::Diff {
                cache_dir,
                key_a,
                key_b,
            } => {
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .ok_or(ExitCode::from(3))?;
                let report = cache_diff(&dir, &key_a, &key_b)?;
                let comparable = report["comparable"].as_bool().unwrap_or(false);
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.diff",
                        true,
                        report,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
                if !comparable {
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
            from_run,
            out,
            manifest_only,
            without_artifacts,
            provenance_only,
            redact,
            with_files,
            include_files,
        } => routes::export_import_routes::handle_export_command(
            &cli,
            run_dir,
            from_run,
            out,
            *manifest_only,
            *without_artifacts,
            *provenance_only,
            *redact,
            *with_files,
            *include_files,
        ),
        Commands::Import { file, verify_only } => {
            routes::export_import_routes::handle_import_command(&cli, file, *verify_only)
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
        Commands::Capabilities { backend } => {
            routes::surface_routes::handle_capabilities_command(&cli, backend)
        }
        Commands::SemanticPortability { backend } => {
            routes::surface_routes::handle_semantic_portability_command(&cli, backend)
        }
        Commands::EquivalenceProof {
            run_a,
            run_b,
            backend_a,
            backend_b,
        } => {
            let diff = replay_service::run_diff_from_dirs(run_a, run_b)?;
            let backend_supported = backend_capability_payload(&backend_a).is_some()
                && backend_capability_payload(&backend_b).is_some();
            let status = if diff.replay_equivalence.equivalent && backend_supported {
                "equivalent"
            } else if backend_supported {
                "fidelity-preserving"
            } else {
                "downgraded"
            };
            let payload = json!({
                "format": "equivalence-proof/v1",
                "backend_a": backend_a,
                "backend_b": backend_b,
                "status": status,
                "run_equivalent": diff.replay_equivalence.equivalent,
                "summary": diff.replay_equivalence.reason_report.summary,
                "reasons": diff.replay_equivalence.reasons
            });
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.equivalence-proof",
                    status != "downgraded",
                    payload,
                    if status == "downgraded" {
                        vec![
                            json!({"message":"equivalence proof downgraded due to unsupported backend or semantic divergence"}),
                        ]
                    } else {
                        Vec::new()
                    },
                    if status == "downgraded" {
                        ExitCode::from(2)
                    } else {
                        ExitCode::SUCCESS
                    },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(if status == "downgraded" {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            })
        }
        Commands::VersionInspect {
            dag,
            run_dir,
            export_bundle,
        } => {
            let provided =
                dag.is_some() as u8 + run_dir.is_some() as u8 + export_bundle.is_some() as u8;
            if provided != 1 {
                return Err(ExitCode::from(2));
            }
            let mut report = json!({
                "binary_version": env!("CARGO_PKG_VERSION"),
                "graph_schema_version": Value::Null,
                "run_dir_format_version": Value::Null,
                "export_bundle_format_version": Value::Null,
            });
            if let Some(path) = dag {
                let input = read_file(path)?;
                let graph = parse_graph(&input)?;
                report["graph_schema_version"] = json!(graph.spec);
                let supported_graph_spec =
                    graph.spec == "0.1" || graph.spec == "v0.1" || graph.spec == SPEC_VERSION;
                if !supported_graph_spec {
                    report["support_status"] = json!("unsupported-graph-schema");
                    if cli.json {
                        return emit_json(
                            &cli,
                            "dag.version-inspect",
                            false,
                            report,
                            vec![
                                json!({"message":"unsupported graph schema version","remediation":"use spec 0.1"} ),
                            ],
                            ExitCode::from(2),
                        );
                    }
                    return Err(ExitCode::from(2));
                }
            }
            if let Some(path) = run_dir {
                let manifest = read_file(&path.join("manifest.json"))?;
                let parsed: Value =
                    serde_json::from_str(&manifest).map_err(|_| ExitCode::from(3))?;
                let run_version = parsed
                    .get("manifest_version")
                    .cloned()
                    .unwrap_or_else(|| json!("run-manifest/v0.1"));
                report["run_dir_format_version"] = run_version.clone();
                report["graph_schema_version"] = parsed.get("spec").cloned().unwrap_or(Value::Null);
                if run_version != json!("run-manifest/v0.1") {
                    report["support_status"] = json!("unsupported-run-dir-version");
                    if cli.json {
                        return emit_json(
                            &cli,
                            "dag.version-inspect",
                            false,
                            report,
                            vec![
                                json!({"message":"unsupported run-dir format version","remediation":"use run-manifest/v0.1"} ),
                            ],
                            ExitCode::from(2),
                        );
                    }
                    return Err(ExitCode::from(2));
                }
            }
            if let Some(path) = export_bundle {
                let payload = read_file(path)?;
                let parsed: Value =
                    serde_json::from_str(&payload).map_err(|_| ExitCode::from(3))?;
                let bundle_version = parsed
                    .get("export_bundle_version")
                    .cloned()
                    .unwrap_or_else(|| json!("export-bundle/v0.1"));
                report["export_bundle_format_version"] = bundle_version.clone();
                report["run_dir_format_version"] = parsed
                    .get("manifest")
                    .and_then(|m| m.get("manifest_version"))
                    .cloned()
                    .unwrap_or(Value::Null);
                report["graph_schema_version"] = parsed
                    .get("manifest")
                    .and_then(|m| m.get("spec"))
                    .cloned()
                    .unwrap_or(Value::Null);
                if bundle_version != json!("export-bundle/v0.1") {
                    report["support_status"] = json!("unsupported-export-bundle-version");
                    if cli.json {
                        return emit_json(
                            &cli,
                            "dag.version-inspect",
                            false,
                            report,
                            vec![
                                json!({"message":"unsupported export bundle version","remediation":"use export-bundle/v0.1"} ),
                            ],
                            ExitCode::from(2),
                        );
                    }
                    return Err(ExitCode::from(2));
                }
            }
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.version-inspect",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        Commands::Config { command } => match command {
            ConfigCommands::ShowEffective {
                config,
                jobs,
                cache_mode,
                materialize_inputs,
            } => {
                let effective = show_effective_config(ShowEffectiveConfigRequest {
                    config_path: config.as_deref(),
                    jobs: *jobs,
                    cache_mode: *cache_mode,
                    materialize_inputs: *materialize_inputs,
                })?;
                let payload = serde_json::to_value(&effective).map_err(|_| ExitCode::from(3))?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.config.show-effective",
                        true,
                        payload,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                Ok(ExitCode::SUCCESS)
            }
        },
        Commands::Policy { command } => match command {
            PolicyCommands::ShowEffective {
                config,
                deny_network,
                deny_env,
                deny_clock,
                clean_env,
                allow_env,
            } => {
                let payload = show_effective_policy(ShowEffectivePolicyRequest {
                    config_path: config.as_deref(),
                    deny_network: *deny_network,
                    deny_env: *deny_env,
                    deny_clock: *deny_clock,
                    clean_env: *clean_env,
                    allow_env,
                })?;
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.policy.show-effective",
                        true,
                        payload,
                        Vec::new(),
                        ExitCode::SUCCESS,
                    );
                }
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                Ok(ExitCode::SUCCESS)
            }
        },
    }
}

pub(crate) fn read_file(path: &Path) -> Result<String, ExitCode> {
    fs::read_to_string(path).map_err(|_| ExitCode::from(3))
}

pub(crate) fn read_run_id(run_dir: &Path) -> Result<String, ExitCode> {
    let raw = read_file(&run_dir.join("manifest.json"))?;
    let value: Value = serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))?;
    value
        .get("run_id")
        .and_then(Value::as_str)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| ExitCode::from(3))
}

fn hash_run_dir(run_dir: &Path) -> Result<String, ExitCode> {
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

pub(crate) fn selector_cli_string(selector: &bijux_dag_runtime::Selector) -> String {
    match selector {
        bijux_dag_runtime::Selector::IdPrefix(v) => format!("id:{v}"),
        bijux_dag_runtime::Selector::Tag(v) => format!("tag:{v}"),
        bijux_dag_runtime::Selector::Kind(v) => format!("kind:{v}"),
    }
}

pub(crate) fn parse_graph(input: &str) -> Result<Graph, ExitCode> {
    match parse_graph_strict(input) {
        Ok(g) => Ok(g),
        Err(GraphError::InvalidSpec(_)) => {
            let mut value = serde_json::from_str::<Value>(input).map_err(|_| ExitCode::from(2))?;
            if let Some(spec) = value.get("spec").and_then(Value::as_str) {
                if spec == "0.1" || spec == "v0.1" {
                    value["spec"] = Value::String(SPEC_VERSION.to_string());
                    let rewritten = serde_json::to_string(&value).map_err(|_| ExitCode::from(2))?;
                    return parse_graph_strict(&rewritten).map_err(|_| ExitCode::from(1));
                }
            }
            Err(ExitCode::from(1))
        }
        Err(GraphError::Json(_)) => Err(ExitCode::from(2)),
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

pub(crate) fn load_snapshot(run_dir: &Path) -> Result<GraphSnapshot, ExitCode> {
    let snap = read_file(&run_dir.join("graph.snapshot.json"))?;
    serde_json::from_str(&snap).map_err(|_| ExitCode::from(3))
}

pub(crate) fn read_node_traces(
    run_dir: &Path,
) -> Result<HashMap<String, serde_json::Value>, ExitCode> {
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

pub(crate) fn read_outputs_indexes(
    run_dir: &Path,
) -> Result<HashMap<String, OutputsIndex>, ExitCode> {
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

pub fn inspect_artifact(run_dir: &Path, artifact_id: &str) -> Result<Value, ExitCode> {
    let (node_id, file_name) = artifact_id
        .split_once(':')
        .ok_or_else(|| ExitCode::from(2))?;
    let manifest_raw = read_file(&run_dir.join("manifest.json"))?;
    let manifest: bijux_dag_artifacts::Manifest =
        serde_json::from_str(&manifest_raw).map_err(|_| ExitCode::from(3))?;
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

pub(crate) fn print_human_diff(diff: &serde_json::Value) {
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

pub(crate) fn collect_output_files(
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

fn verify_cache_dirs(local: &Path, remote: Option<&Path>) -> Result<serde_json::Value, ExitCode> {
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

fn explain_cache_key(
    cache_dir: &Path,
    key: &str,
    expected_adapter_id: &str,
    expected_adapter_version: &str,
) -> Result<Value, ExitCode> {
    let entry = cache_dir.join(key);
    let mut reasons = Vec::new();
    if !entry.exists() {
        reasons.push("missing cache entry directory".to_string());
        return Ok(json!({
            "key": key,
            "eligible": false,
            "reasons": reasons
        }));
    }
    let meta_path = entry.join("meta.json");
    let index_path = entry.join("outputs").join("index.json");
    if !meta_path.exists() {
        reasons.push("missing meta.json".to_string());
    }
    if !index_path.exists() {
        reasons.push("missing outputs/index.json".to_string());
    }
    let mut meta = Value::Null;
    if meta_path.exists() {
        meta = serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(key) {
            reasons.push("node_fingerprint mismatch".to_string());
        }
        if !expected_adapter_id.is_empty()
            && meta.get("adapter_id").and_then(|v| v.as_str()) != Some(expected_adapter_id)
        {
            reasons.push("adapter_id mismatch".to_string());
        }
        if !expected_adapter_version.is_empty()
            && meta.get("adapter_version").and_then(|v| v.as_str())
                != Some(expected_adapter_version)
        {
            reasons.push("adapter_version mismatch".to_string());
        }
    }
    let eligible = reasons.is_empty()
        && verify_cache_entry_cli(
            entry.as_path(),
            key,
            expected_adapter_id,
            expected_adapter_version,
        )?;
    if !eligible && reasons.is_empty() {
        reasons.push("output proof verification failed".to_string());
    }
    Ok(json!({
        "key": key,
        "eligible": eligible,
        "entry_dir": entry,
        "meta": meta,
        "reasons": reasons
    }))
}

fn cache_stats(cache_dir: &Path) -> Result<Value, ExitCode> {
    if !cache_dir.exists() {
        return Ok(json!({
            "entries": 0,
            "bytes": 0u64,
            "invalid_entries": 0,
            "hit_potential": "none"
        }));
    }
    let mut entries = 0u64;
    let mut bytes = 0u64;
    let mut invalid_entries = 0u64;
    for dirent in fs::read_dir(cache_dir).map_err(|_| ExitCode::from(3))? {
        let dirent = dirent.map_err(|_| ExitCode::from(3))?;
        if !dirent.path().is_dir() {
            continue;
        }
        entries += 1;
        let key = dirent.file_name().to_string_lossy().to_string();
        let path = dirent.path();
        let valid = verify_cache_entry_cli(&path, &key, "", "")?;
        if !valid {
            invalid_entries += 1;
        }
        bytes += dir_size_bytes(&path)?;
    }
    let hit_potential = if entries == 0 {
        "none"
    } else if invalid_entries == 0 {
        "high"
    } else if invalid_entries * 2 < entries {
        "medium"
    } else {
        "low"
    };
    Ok(json!({
        "entries": entries,
        "bytes": bytes,
        "invalid_entries": invalid_entries,
        "hit_potential": hit_potential
    }))
}

fn cache_prune_simulate(cache_dir: &Path) -> Result<Value, ExitCode> {
    if !cache_dir.exists() {
        return Ok(json!({"would_remove": [], "reason": "cache directory missing"}));
    }
    let mut would_remove = Vec::new();
    for dirent in fs::read_dir(cache_dir).map_err(|_| ExitCode::from(3))? {
        let dirent = dirent.map_err(|_| ExitCode::from(3))?;
        if !dirent.path().is_dir() {
            continue;
        }
        let key = dirent.file_name().to_string_lossy().to_string();
        let valid = verify_cache_entry_cli(&dirent.path(), &key, "", "")?;
        if !valid {
            would_remove.push(key);
        }
    }
    would_remove.sort();
    Ok(json!({
        "would_remove": would_remove,
        "policy": "invalid entries only (simulation)"
    }))
}

fn cache_diff(cache_dir: &Path, key_a: &str, key_b: &str) -> Result<Value, ExitCode> {
    fn load_meta(entry: &Path) -> Result<Value, ExitCode> {
        let meta_path = entry.join("meta.json");
        if !meta_path.exists() {
            return Ok(json!({}));
        }
        serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))
    }
    let a_path = cache_dir.join(key_a);
    let b_path = cache_dir.join(key_b);
    let a_exists = a_path.exists();
    let b_exists = b_path.exists();
    if !a_exists || !b_exists {
        return Ok(json!({
            "key_a": key_a,
            "key_b": key_b,
            "comparable": false,
            "reason": "missing cache entry",
            "missing": {
                "key_a": !a_exists,
                "key_b": !b_exists
            }
        }));
    }
    let a_meta = load_meta(&a_path)?;
    let b_meta = load_meta(&b_path)?;
    let mut differences = Vec::new();
    for field in [
        "node_fingerprint",
        "adapter_id",
        "adapter_version",
        "output_schema_version",
        "policy_fingerprint",
        "config_fingerprint",
        "backend_class",
        "cache_metadata_version",
        "source_run_id",
        "cache_source",
    ] {
        if a_meta.get(field) != b_meta.get(field) {
            differences.push(json!({
                "field": field,
                "a": a_meta.get(field).cloned().unwrap_or(Value::Null),
                "b": b_meta.get(field).cloned().unwrap_or(Value::Null),
            }));
        }
    }
    let a_valid = verify_cache_entry_cli(&a_path, key_a, "", "")?;
    let b_valid = verify_cache_entry_cli(&b_path, key_b, "", "")?;
    Ok(json!({
        "key_a": key_a,
        "key_b": key_b,
        "comparable": true,
        "valid": {
            "key_a": a_valid,
            "key_b": b_valid
        },
        "differences": differences
    }))
}

fn dir_size_bytes(path: &Path) -> Result<u64, ExitCode> {
    let mut total = 0u64;
    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let p = entry.path();
        if p.is_dir() {
            total += dir_size_bytes(&p)?;
        } else {
            total += fs::metadata(&p).map_err(|_| ExitCode::from(3))?.len();
        }
    }
    Ok(total)
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

fn verify_run(run_dir: &Path, deep: bool, strict: bool) -> Result<serde_json::Value, ExitCode> {
    let mut errors = Vec::new();
    let mut invariant_violations = Vec::new();
    let manifest_path = run_dir.join("manifest.json");
    let manifest_data = fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?;
    let manifest: bijux_dag_artifacts::Manifest =
        serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
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
            let val: serde_json::Value =
                serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
            if let Some(status) = val.get("status").and_then(|s| s.as_str()) {
                match status {
                    "success" => observed_statuses.push(bijux_dag_runtime::NodeStatus::Success),
                    "failed" => observed_statuses.push(bijux_dag_runtime::NodeStatus::Failed),
                    "skipped" => observed_statuses.push(bijux_dag_runtime::NodeStatus::Skipped),
                    "cached" => observed_statuses.push(bijux_dag_runtime::NodeStatus::Cached),
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
                if !bijux_dag_runtime::invariants::trace_time_order_ok(started, finished) {
                    invariant_violations.push(format!(
                        "INV-TRACE-TIME-001 violation in {}",
                        entry.file_name().to_string_lossy()
                    ));
                }
            }
        }
    }

    let manifest_counts = bijux_dag_runtime::invariants::RunNodeCounts {
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
    };
    if !bijux_dag_runtime::invariants::run_summary_invariant_ok(manifest_counts, &observed_statuses)
    {
        invariant_violations
            .push("INV-RUN-COUNTS-001 manifest totals do not match node traces".to_string());
    }
    if manifest.status == "completed"
        && !bijux_dag_runtime::invariants::terminal_run_has_terminal_node(&observed_statuses)
    {
        invariant_violations
            .push("INV-RUN-TERMINAL-001 completed run has no terminal node statuses".to_string());
    }

    if deep || strict {
        if serde_json::from_str::<bijux_dag_artifacts::Manifest>(&manifest_data).is_err() {
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
                errors.push(format!("strict verify missing required run artifact: {}", rel));
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
        let manifest_json: Value = serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
        if let Some(summary) = manifest_json
            .get("run_metadata")
            .and_then(|m| m.get("environment_summary"))
        {
            let summary_bytes =
                serde_json::to_vec(summary).map_err(|_| ExitCode::from(3))?;
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

pub(crate) fn map_materialize_mode(arg: MaterializeModeArg) -> MaterializeMode {
    match arg {
        MaterializeModeArg::Copy => MaterializeMode::Copy,
        MaterializeModeArg::Hardlink => MaterializeMode::Hardlink,
        MaterializeModeArg::Symlink => MaterializeMode::Symlink,
    }
}

fn collect_json_diff_paths(path: &str, left: &Value, right: &Value, out: &mut Vec<String>) {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys = std::collections::BTreeSet::new();
            keys.extend(a.keys().cloned());
            keys.extend(b.keys().cloned());
            for key in keys {
                let child = if path.is_empty() {
                    format!("/{}", key)
                } else {
                    format!("{}/{}", path, key)
                };
                match (a.get(&key), b.get(&key)) {
                    (Some(lv), Some(rv)) => collect_json_diff_paths(&child, lv, rv, out),
                    _ => out.push(child),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            let max = a.len().max(b.len());
            for idx in 0..max {
                let child = if path.is_empty() {
                    format!("/{}", idx)
                } else {
                    format!("{}/{}", path, idx)
                };
                match (a.get(idx), b.get(idx)) {
                    (Some(lv), Some(rv)) => collect_json_diff_paths(&child, lv, rv, out),
                    _ => out.push(child),
                }
            }
        }
        _ => {
            if left != right {
                out.push(if path.is_empty() {
                    "/".to_string()
                } else {
                    path.to_string()
                });
            }
        }
    }
}

pub(crate) fn verify_bundle_invariants(bundle: &serde_json::Value) -> Vec<String> {
    let mut violations = Vec::new();
    if bundle.get("bundle_version").and_then(|v| v.as_str()) != Some("export-bundle/v0.1") {
        violations.push("INV-EXPORT-VERSION-001 unsupported or missing bundle_version".to_string());
    }
    match bundle.get("export_mode").and_then(|v| v.as_str()) {
        Some("manifest-only")
        | Some("with-files")
        | Some("without-artifacts")
        | Some("provenance-only") => {}
        _ => violations.push("INV-EXPORT-MODE-001 unsupported or missing export_mode".to_string()),
    }
    if bundle.get("manifest").is_none() {
        violations.push("INV-EXPORT-VERIFY-001 missing manifest".to_string());
    }
    if bundle.get("graph_snapshot").is_none() {
        violations.push("INV-EXPORT-VERIFY-001 missing graph_snapshot".to_string());
    }
    if bundle
        .get("node_traces")
        .and_then(|v| v.as_object())
        .is_none()
    {
        violations.push("INV-EXPORT-VERIFY-001 missing node_traces map".to_string());
    }
    if bundle.get("outputs").and_then(|v| v.as_object()).is_none() {
        violations.push("INV-EXPORT-VERIFY-001 missing outputs map".to_string());
    }
    let files = bundle.get("files");
    if bundle.get("export_mode").and_then(|v| v.as_str()) == Some("manifest-only")
        && !matches!(files, None | Some(serde_json::Value::Null))
    {
        violations.push(
            "INV-EXPORT-MODE-001 manifest-only bundle must not include files payload".to_string(),
        );
    }
    if bundle.get("export_mode").and_then(|v| v.as_str()) == Some("with-files")
        && !files.is_some_and(|v| v.is_object())
    {
        violations.push("INV-EXPORT-MODE-001 with-files bundle must include files map".to_string());
    }
    if let Some(files_map) = files.and_then(|v| v.as_object()) {
        for (node_id, node_files) in files_map {
            let Some(node_files) = node_files.as_object() else {
                violations.push(format!(
                    "INV-EXPORT-FILES-001 files entry for node {node_id} must be object"
                ));
                continue;
            };
            for (path, encoded) in node_files {
                if encoded.as_str().is_none() {
                    violations.push(format!(
                        "INV-EXPORT-FILES-001 file payload for {node_id}/{path} must be base64 string"
                    ));
                    continue;
                }
                let value = encoded.as_str().unwrap_or_default();
                if BASE64.decode(value).is_err() {
                    violations.push(format!(
                        "INV-EXPORT-FILES-001 file payload for {node_id}/{path} is not valid base64"
                    ));
                }
            }
        }
    }
    if bundle.get("export_mode").and_then(|v| v.as_str()) == Some("without-artifacts") {
        if !bundle
            .get("outputs")
            .is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty()))
        {
            violations.push(
                "INV-EXPORT-MODE-001 without-artifacts bundle must include empty outputs map"
                    .to_string(),
            );
        }
        if !matches!(files, None | Some(serde_json::Value::Null)) {
            violations.push(
                "INV-EXPORT-MODE-001 without-artifacts bundle must not include files payload"
                    .to_string(),
            );
        }
    }
    if bundle.get("export_mode").and_then(|v| v.as_str()) == Some("provenance-only") {
        if !bundle
            .get("node_traces")
            .is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty()))
        {
            violations.push(
                "INV-EXPORT-MODE-001 provenance-only bundle must include empty node_traces map"
                    .to_string(),
            );
        }
        if !bundle
            .get("outputs")
            .is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty()))
        {
            violations.push(
                "INV-EXPORT-MODE-001 provenance-only bundle must include empty outputs map"
                    .to_string(),
            );
        }
    }
    if let Some(traces) = bundle.get("node_traces").and_then(|v| v.as_object()) {
        for (node_id, trace) in traces {
            let trace_node_id = trace.get("node_id").and_then(|v| v.as_str());
            if trace_node_id != Some(node_id.as_str()) {
                violations.push(format!(
                    "INV-TRACE-ATTEMPT-001 node_id mismatch for trace key {}",
                    node_id
                ));
            }
            if trace.get("status").and_then(|v| v.as_str()).is_none() {
                violations.push(format!(
                    "INV-TRACE-ATTEMPT-001 missing status for trace key {}",
                    node_id
                ));
            }
        }
    }
    violations
}

pub(crate) fn build_run_proof_bundle(run_dir: &Path) -> Result<serde_json::Value, ExitCode> {
    let report = verify_run(run_dir, true, true)?;
    let status = report
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("error")
        .to_string();
    let errors = report
        .get("errors")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let invariant_violations = report
        .get("invariant_violations")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let has_manifest = run_dir.join("manifest.json").exists();
    let has_snapshot = run_dir.join("graph.snapshot.json").exists();
    let has_outputs = run_dir.join("outputs").join("index.json").exists();
    let run_id = read_run_id(run_dir).unwrap_or_else(|_| "unknown".to_string());
    let proof_id = format!("proof-{}", run_id);

    let mut incomplete_reasons: Vec<String> = Vec::new();
    if !has_manifest {
        incomplete_reasons.push("missing manifest".to_string());
    }
    if !has_snapshot {
        incomplete_reasons.push("missing graph snapshot".to_string());
    }
    if !has_outputs {
        incomplete_reasons.push("missing outputs index".to_string());
    }
    if !errors.is_empty() {
        incomplete_reasons.push("verification errors present".to_string());
    }
    if !invariant_violations.is_empty() {
        incomplete_reasons.push("invariant violations present".to_string());
    }

    let provenance_path = run_dir.join("provenance.json");
    let backend_origin = if provenance_path.exists() {
        let raw = read_file(&provenance_path).unwrap_or_default();
        let value: Value = serde_json::from_str(&raw).unwrap_or_default();
        value
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("native-run")
            .to_string()
    } else {
        "native-run".to_string()
    };

    let complete = incomplete_reasons.is_empty() && status == "ok";
    Ok(json!({
        "schema_version": "proof-bundle/v0.1",
        "proof_id": proof_id,
        "run_id": run_id,
        "run_dir": run_dir,
        "backend_origin": backend_origin,
        "status": if complete { "complete" } else { "incomplete" },
        "complete": complete,
        "determinism": if complete { "verified" } else { "insufficient-evidence" },
        "integrity": if complete { "verified" } else { "insufficient-evidence" },
        "replay_evidence": {
            "available": has_manifest && has_snapshot,
            "level": if complete { "complete" } else { "partial" }
        },
        "integrity_evidence": {
            "available": has_outputs,
            "level": if complete { "complete" } else { "partial" }
        },
        "incomplete_reasons": incomplete_reasons,
        "verification_errors": errors,
        "invariant_violations": invariant_violations,
        "signing": {
            "signed": false,
            "signature_format": Value::Null,
            "signature": Value::Null,
            "trust_level": "unsigned"
        }
    }))
}

#[cfg(test)]
mod invariant_bundle_tests {
    use super::verify_bundle_invariants;
    use serde_json::json;

    #[test]
    fn bundle_invariants_accept_well_formed_bundle() {
        let bundle = json!({
            "bundle_version":"export-bundle/v0.1",
            "export_mode":"manifest-only",
            "manifest": {"status":"completed"},
            "graph_snapshot": {"nodes":[],"edges":[]},
            "node_traces": {
                "n1": {"node_id":"n1","status":"success"}
            },
            "outputs": {},
            "files": null
        });
        let violations = verify_bundle_invariants(&bundle);
        assert!(violations.is_empty());
    }

    #[test]
    fn bundle_invariants_reject_missing_and_incoherent_fields() {
        let bundle = json!({
            "graph_snapshot": {},
            "node_traces": {
                "n1": {"node_id":"n2"}
            }
        });
        let violations = verify_bundle_invariants(&bundle);
        assert!(!violations.is_empty());
        assert!(violations
            .iter()
            .any(|v| v.contains("INV-EXPORT-VERIFY-001")));
        assert!(violations
            .iter()
            .any(|v| v.contains("INV-TRACE-ATTEMPT-001")));
    }
}
