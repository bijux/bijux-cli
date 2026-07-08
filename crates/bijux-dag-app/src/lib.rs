//! Application orchestration and response shaping for the `bijux-dag` command surface.
//!
//! Prefer [`stable`] when browsing the long-lived app surface, [`prelude`] for
//! command embedding helpers, and crate-root imports only when you already
//! know the exact item you need. Broad compatibility re-exports remain
//! callable for focused imports, but they are intentionally hidden from the
//! default docs lane. The `experimental-public-api` feature enables
//! repository-owned contract helpers that are intentionally excluded from the
//! default docs lane.
//!
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
#[path = "inspect/integrity_service.rs"]
mod integrity_service;
#[cfg(feature = "experimental-public-api")]
#[path = "commands/iteration08_contracts.rs"]
mod iteration08_contracts;
#[cfg(feature = "experimental-public-api")]
#[path = "commands/iteration11_contracts.rs"]
mod iteration11_contracts;
mod migrate;
#[path = "inspect/node_execution_explanation.rs"]
mod node_execution_explanation;
#[path = "commands/output_contract.rs"]
mod output_contract;
mod read;
#[path = "read/read_graph.rs"]
mod read_graph;
#[path = "commands/reference_docs.rs"]
mod reference_docs;
mod repair;
#[path = "repair/service.rs"]
mod repair_service;
mod replay;
#[path = "replay/cmd.rs"]
mod replay_cmd;
#[path = "replay/service.rs"]
mod replay_service;
mod routes;
#[path = "commands/run_cmd.rs"]
mod run_cmd;
#[path = "inspect/run_comparison.rs"]
mod run_comparison;
#[path = "read/run_data.rs"]
mod run_data;
#[path = "inspect/run_failure_summary.rs"]
mod run_failure_summary;
#[path = "inspect/run_views.rs"]
mod run_views;
#[path = "read/runtime_inputs.rs"]
mod runtime_inputs;
#[path = "inspect/status_cmd.rs"]
mod status_cmd;
#[path = "graph/validate_cmd.rs"]
mod validate_cmd;
mod write;

#[doc(hidden)]
pub use config_surface::{
    config_fingerprint, default_runtime_config, normalize_runtime_config, policy_evaluation_trace,
    resolve_effective_config, CacheModeSurface, MaterializeInputsSurface,
    PartialRuntimeSurfaceConfig, PolicySurfaceConfig, RuntimeSurfaceConfig,
};
#[doc(hidden)]
pub use integrity_service::inspect_artifact;
#[doc(hidden)]
pub use reference_docs::write_checked_in_cli_reference_docs;
#[doc(hidden)]
pub use run_comparison::runs_compare;
#[doc(hidden)]
pub use run_failure_summary::explain_failure;
#[doc(hidden)]
pub use run_views::{
    doctor_run, explain_run_id, format_inspect_human, format_run_completion_human,
    format_show_human, inspect_summary, list_runs, resolve_run_dir, run_completion_summary,
    run_scheduler_checkpoint, run_timeline, run_tree, runs_failures, runs_flakes, runs_history,
    runs_history_query, runs_summary, runs_trend,
};

/// Explicit long-lived command embedding and response-shaping surface.
pub mod stable {
    pub use crate::{
        dag_command, dag_run, default_runtime_config, inspect_artifact, list_runs,
        normalize_runtime_config, policy_evaluation_trace, resolve_effective_config,
        resolve_run_dir, runs_summary, CacheModeSurface, MaterializeInputsSurface,
        PartialRuntimeSurfaceConfig, PolicySurfaceConfig, RuntimeSurfaceConfig,
    };
}

/// Common imports for embedding `bijux-dag` command orchestration.
pub mod prelude {
    pub use crate::stable::{
        dag_command, dag_run, default_runtime_config, inspect_artifact, normalize_runtime_config,
        resolve_effective_config, RuntimeSurfaceConfig,
    };
}

/// Opt-in app contract helpers that are outside the stable command lane.
#[cfg(feature = "experimental-public-api")]
pub mod experimental {
    pub mod command_reports {
        pub use crate::iteration08_contracts::*;
    }
    pub mod workspace_compatibility {
        pub use crate::iteration11_contracts::*;
    }
}

use crate::cache::{
    cache_diff, cache_prune_simulate, cache_stats, explain_cache_key, explain_run_node_cache_miss,
    pack_cache_entry, unpack_cache_entry, verify_cache_dirs,
};
use crate::cli_model::command_name as dag_command_name;
use crate::integrity_service::{check_engine, hash_run_dir, verify_run};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_core::{Graph, GraphError, Severity, SPEC_VERSION};
use bijux_dag_runtime::{CacheMode, Runtime, RuntimeConfig};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use commands::{
    command_access_denial, hide_non_public_help, lane_label, CacheCommands, CommandAccessDenial,
    Commands, ConfigCommands, DagCli, GraphFormatArg, HashCommands, MigrateCommands,
    PolicyCommands,
};
use config_resolution::{
    show_effective_config, show_effective_policy, ShowEffectiveConfigRequest,
    ShowEffectivePolicyRequest,
};
use graph_helpers::*;
use output_contract::{emit_json, LintDiagnostic};
use run_data::env_cache_dir;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use thiserror as _;

pub fn dag_command() -> clap::Command {
    let command = DagCli::command().name(dag_command_name()).subcommand_required(false);
    hide_non_public_help(command, "")
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

fn run(cli: DagCli) -> Result<ExitCode, ExitCode> {
    if let Some(denial) = command_access_denial(&cli.command) {
        return emit_command_access_denial(&cli, denial);
    }
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
        Commands::Validate { dags, strict, print_fingerprints, explain } => {
            routes::validate_routes::handle_validate_command(
                &cli,
                dags,
                *strict,
                *print_fingerprints,
                *explain,
            )
        }
        Commands::Canonicalize { dags } => {
            let graph = load_graphs_or_emit(&cli, "dag.canonicalize", dags)?;
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
        Commands::Lint { dags, strict } => {
            let strict = *strict;
            let graph = load_graphs_or_emit(&cli, "dag.lint", dags)?;
            let lint = lint_graph(&graph);
            let has_warnings = !lint.is_empty();
            if cli.json {
                let diagnostics: Vec<Value> =
                    lint.iter().map(|d| serde_json::to_value(d).unwrap()).collect();
                let code =
                    if strict && has_warnings { ExitCode::from(2) } else { ExitCode::SUCCESS };
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
        Commands::GraphLint { dags, strict } => {
            let graph = load_graphs_or_emit(&cli, "dag.graph-lint", dags)?;
            let lint = lint_graph(&graph);
            let has_warnings = !lint.is_empty();
            if cli.json {
                let diagnostics: Vec<Value> =
                    lint.iter().map(|d| serde_json::to_value(d).unwrap()).collect();
                let code =
                    if *strict && has_warnings { ExitCode::from(2) } else { ExitCode::SUCCESS };
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
        Commands::Fingerprint { dags, explain } => {
            let graph = load_graphs_or_emit(&cli, "dag.fingerprint", dags)?;
            let explained = graph.graph_fingerprint_explain().map_err(|_| ExitCode::from(3))?;
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
                    println!("canonical_json_bytes_len={}", explained.canonical_json_bytes_len);
                } else {
                    println!("{}", explained.graph_id.as_str());
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Hash { command } => match command {
            HashCommands::Graph { dags, explain } => {
                let graph = load_graphs_or_emit(&cli, "dag.hash.graph", dags)?;
                let explained = graph.graph_fingerprint_explain().map_err(|_| ExitCode::from(3))?;
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
                    println!("canonical_json_bytes_len={}", explained.canonical_json_bytes_len);
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
        Commands::ArtifactInspect { run_dir, artifact_id } => {
            routes::artifact_routes::handle_artifact_inspect_command(&cli, run_dir, artifact_id)
        }
        Commands::Artifact { command } => {
            routes::artifact_routes::handle_artifact_command(&cli, command)
        }
        Commands::ControlPlane { command } => {
            routes::control_plane_routes::handle_control_plane_command(&cli, command)
        }
        Commands::StateStore { command } => {
            routes::state_store_routes::handle_state_store_command(&cli, command)
        }
        Commands::Dataset { command } => {
            routes::dataset_routes::handle_dataset_command(&cli, command)
        }
        Commands::Enterprise { command } => {
            routes::enterprise_routes::handle_enterprise_command(&cli, command)
        }
        Commands::Fleet { command } => routes::fleet_routes::handle_fleet_command(&cli, command),
        Commands::Governance { command } => {
            routes::governance_routes::handle_governance_command(&cli, command)
        }
        Commands::Incident { command } => {
            routes::incident_routes::handle_incident_command(&cli, command)
        }
        Commands::Lab { command } => match command {
            commands::LabCommands::Federation { command } => {
                routes::federation_routes::handle_federation_command(&cli, command)
            }
            commands::LabCommands::Incident { command } => {
                routes::incident_routes::handle_incident_command(&cli, command)
            }
            commands::LabCommands::Enterprise { command } => {
                routes::enterprise_routes::handle_enterprise_command(&cli, command)
            }
            commands::LabCommands::Release { command } => {
                routes::release_routes::handle_release_command(&cli, command)
            }
            commands::LabCommands::Security { command } => {
                routes::security_routes::handle_security_command(&cli, command)
            }
            commands::LabCommands::Durability { command } => {
                routes::durability_routes::handle_durability_command(&cli, command)
            }
            commands::LabCommands::Performance { command } => {
                routes::performance_routes::handle_performance_command(&cli, command)
            }
        },
        Commands::Federation { command } => {
            routes::federation_routes::handle_federation_command(&cli, command)
        }
        Commands::Security { command } => {
            routes::security_routes::handle_security_command(&cli, command)
        }
        Commands::Durability { command } => {
            routes::durability_routes::handle_durability_command(&cli, command)
        }
        Commands::Performance { command } => {
            routes::performance_routes::handle_performance_command(&cli, command)
        }
        Commands::Release { command } => {
            routes::release_routes::handle_release_command(&cli, command)
        }
        Commands::CanonicalBytes { dags } => {
            let graph = load_graphs_or_emit(&cli, "dag.canonical-bytes", dags)?;
            let bytes = graph.canonical_json_bytes().map_err(|_| ExitCode::from(3))?;
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
        Commands::ShowEffectiveGraph {
            dags,
            run_dir,
            select,
            exclude,
            from_node,
            to_node,
            dependency_closure,
        } => routes::graph_routes::handle_show_effective_graph_command(
            &cli,
            dags,
            run_dir,
            select,
            exclude,
            from_node,
            to_node,
            *dependency_closure,
        ),
        Commands::ExplainPlan {
            dags,
            out,
            run_id,
            cache_dir,
            absolute_path_policy,
            jobs,
            cpu_budget,
            memory_budget_mb,
            gpu_device_budget,
            resource_capacity,
            from_node,
            to_node,
        } => {
            let graph = load_graphs_or_emit(&cli, "dag.explain-plan", dags)?;
            graph_helpers::validate_partial_selection_surface(from_node, to_node, &[], &[], false)?;
            let (upstream_selection_targets, _) =
                graph_helpers::resolve_upstream_run_selection(&graph, to_node)?;
            let (downstream_selection_roots, _) =
                graph_helpers::resolve_downstream_run_selection(&graph, from_node)?;
            let preview_layout = routes::plan_routes::resolve_plan_preview_layout(
                out.as_deref(),
                run_id.as_deref(),
            )?;
            let named_resource_capacities =
                routes::resource_capacity_args::parse_resource_capacities(resource_capacity)?;
            let preview = routes::plan_routes::PlanPreviewConfig {
                run_root: out.clone(),
                run_id: preview_layout.as_ref().map(|layout| layout.run_id.clone()),
                cache_dir: cache_dir.clone(),
                absolute_path_policy: (*absolute_path_policy).into(),
                jobs: *jobs,
                cpu_budget: *cpu_budget,
                memory_budget_mb: *memory_budget_mb,
                gpu_device_budget: *gpu_device_budget,
                named_resource_capacities,
                upstream_selection_targets,
                downstream_selection_roots,
                selectors: bijux_dag_runtime::SelectorSet::default(),
                dependency_closure: false,
            };
            let analysis = routes::plan_routes::build_default_planner_analysis(&graph, &preview)
                .map_err(|_| ExitCode::from(3))?;
            let payload = routes::plan_routes::plan_explain_payload(
                &analysis,
                preview_layout.as_ref(),
                preview.absolute_path_policy,
            );
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.explain-plan",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for line in routes::plan_routes::concise_plan_lines(&analysis) {
                println!("{line}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Plan { command } => routes::plan_routes::handle_plan_command(&cli, command),
        Commands::Schedule { command } => {
            routes::schedule_routes::handle_schedule_command(&cli, command)
        }
        Commands::Runtime { command } => {
            routes::runtime_routes::handle_runtime_command(&cli, command)
        }
        Commands::Graph { dags, format } => {
            let graph = load_graphs_or_emit(&cli, "dag.graph", dags)?;
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
        Commands::Replay { command } => routes::replay_routes::handle_replay_command(
            &cli,
            command.run_dir.as_deref(),
            command.source_run_id.as_deref(),
            command.source_run_root.as_deref(),
            &command.out,
            command.dry_run,
            command.sandbox,
            command.prove,
            command.reuse_cache,
            command.cache,
            command.jobs,
            command.run_id.clone(),
            command.cpu_budget,
            command.memory_budget_mb,
            command.gpu_device_budget,
            &command.resource_capacity,
            command.deny_network,
            command.deny_env,
            command.deny_clock,
            command.clean_env,
            command.hermetic,
            &command.from_node,
            &command.select,
            &command.exclude,
            command.dependency_closure,
            command.materialize_inputs,
            command.remote_cache_dir.clone(),
        ),
        Commands::Prove { run_dir } => {
            routes::prove_verify_routes::handle_prove_command(&cli, run_dir)
        }
        Commands::ProofSummary { run_dir } => {
            routes::prove_verify_routes::handle_proof_summary_command(&cli, run_dir)
        }
        Commands::Runs { command } => routes::runs_routes::handle_runs_command(&cli, command),
        Commands::Diff { run_a, run_b, mode, node, explain } => {
            routes::diff_routes::handle_diff_command(
                &cli,
                run_a,
                run_b,
                *mode,
                node.as_deref(),
                *explain,
                "dag.diff",
            )
        }
        Commands::WhyRerun { run_a, run_b, node } => {
            routes::diagnostics_routes::handle_why_rerun_command(
                &cli,
                run_a,
                run_b,
                node.as_deref(),
            )
        }
        Commands::WhyCacheMissed {
            key,
            expected_adapter_id,
            expected_adapter_version,
            run_dir,
            node,
            cache_dir,
        } => {
            let payload = if let (Some(run_dir), Some(node_id)) =
                (run_dir.as_ref(), node.as_deref())
            {
                explain_run_node_cache_miss(run_dir, node_id, cache_dir.as_deref())?
            } else {
                let key = key.as_deref().ok_or(ExitCode::from(3))?;
                let expected_adapter_id =
                    expected_adapter_id.as_deref().ok_or(ExitCode::from(3))?;
                let expected_adapter_version =
                    expected_adapter_version.as_deref().ok_or(ExitCode::from(3))?;
                let dir = cache_dir
                    .clone()
                    .or_else(env_cache_dir)
                    .unwrap_or_else(|| PathBuf::from(".bijux/cache"));
                let report =
                    explain_cache_key(&dir, key, expected_adapter_id, expected_adapter_version)?;
                json!({
                    "mode": "key",
                    "cache_dir": dir,
                    "key": key,
                    "eligible": report["eligible"],
                    "reasons": report["reasons"],
                    "taxonomy": report["taxonomy"],
                    "key_components": report["key_components"],
                    "proof_verified": report["proof_verified"],
                    "meta": report["meta"]
                })
            };
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
        Commands::TraceArtifact { run_dir, artifact_id } => {
            routes::diagnostics_routes::handle_trace_artifact_command(&cli, run_dir, artifact_id)
        }
        Commands::TraceNode { run_dir, id } => {
            routes::diagnostics_routes::handle_trace_node_command(&cli, run_dir, id)
        }
        Commands::Run { command } => routes::run_routes::handle_run_command(
            &cli,
            routes::run_routes::RunRouteRequest {
                dags: &command.dags,
                out: &command.out,
                input: &command.input,
                inputs_file: command.inputs_file.clone(),
                run_id: command.run_id.clone(),
                resume_run: command.resume_run.clone(),
                resume_failure_mode: command.resume_failure_mode,
                latest: command.latest.clone(),
                jobs: command.jobs,
                cpu_budget: command.cpu_budget,
                memory_budget_mb: command.memory_budget_mb,
                gpu_device_budget: command.gpu_device_budget,
                resource_capacity: &command.resource_capacity,
                node_timeout_ms: command.node_timeout_ms,
                run_timeout_ms: command.run_timeout_ms,
                run_timeout_behavior: command.run_timeout_behavior,
                deny_network: command.deny_network,
                deny_env: command.deny_env,
                deny_clock: command.deny_clock,
                clean_env: command.clean_env,
                hermetic: command.hermetic,
                select: &command.select,
                exclude: &command.exclude,
                to_node: &command.to_node,
                dependency_closure: command.dependency_closure,
                materialize_inputs: command.materialize_inputs,
                cache: command.cache,
                cache_dir: command.cache_dir.clone(),
                remote_cache_dir: command.remote_cache_dir.clone(),
                absolute_path_policy: command.absolute_path_policy,
                preflight_only: command.preflight_only,
                explain_scheduling: command.explain_scheduling,
                progress: command.progress,
                backend: command.backend,
                kubernetes_namespace: command.kubernetes_namespace.clone(),
                kubernetes_volume_claim: command.kubernetes_volume_claim.clone(),
                kubernetes_shared_root: command.kubernetes_shared_root.clone(),
                slurm_queue: command.slurm_queue.clone(),
                slurm_partition: command.slurm_partition.clone(),
            },
        ),
        Commands::RunBundle { run_dir, out, redact } => {
            routes::export_import_routes::handle_export_command(
                &cli,
                &Some(run_dir.clone()),
                &None,
                out,
                false,
                false,
                false,
                *redact,
                true,
                false,
            )
        }
        Commands::Explain { run_dir, node } => {
            routes::inspect_routes::handle_explain_command(&cli, run_dir, node)
        }
        Commands::Node { run_dir, id: node } => {
            routes::inspect_routes::handle_node_command(&cli, run_dir, node)
        }
        Commands::Status { run_dir } => {
            routes::inspect_routes::handle_status_command(&cli, run_dir)
        }
        Commands::Verify { run_dir, deep, strict } => {
            routes::prove_verify_routes::handle_verify_command(&cli, run_dir, *deep, *strict)
        }
        Commands::Fsck { run_dir, strict } => {
            routes::prove_verify_routes::handle_fsck_command(&cli, run_dir, *strict)
        }
        Commands::Doctor => {
            let report = doctor_report()?;
            let ok =
                report.get("status").and_then(|v| v.as_str()).map(|v| v == "ok").unwrap_or(false);
            if cli.json {
                return emit_json(
                    &cli,
                    "dag.doctor",
                    ok,
                    report,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            } else {
                println!("status: {}", report["status"]);
            }
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::CommandCatalog { groups, lanes } => {
            routes::command_routes::handle_command_catalog_command(&cli, *groups, lanes)
        }
        Commands::Migrate { command } => {
            let msg = match command {
                MigrateCommands::Dag { file, from, to, dry_run } => {
                    let result = migrate_dag(file, from, to)?;
                    if *dry_run {
                        format!("dry-run: {result}")
                    } else {
                        result
                    }
                }
                MigrateCommands::Run { run_dir, from, to, dry_run } => {
                    let result = migrate_run(run_dir, from, to)?;
                    if *dry_run {
                        format!("dry-run: {result}")
                    } else {
                        result
                    }
                }
                MigrateCommands::Inspect { dag, run_dir, from, to } => {
                    let report = match (dag, run_dir) {
                        (Some(path), None) => inspect_migrate_dag(path, from, to)?,
                        (None, Some(path)) => inspect_migrate_run(path, from, to)?,
                        _ => return Err(ExitCode::from(2)),
                    };
                    if cli.json {
                        return emit_json(
                            &cli,
                            "dag.migrate.inspect",
                            true,
                            report,
                            Vec::new(),
                            ExitCode::SUCCESS,
                        );
                    }
                    println!("{}", serde_json::to_string_pretty(&report).unwrap());
                    return Ok(ExitCode::SUCCESS);
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
            CacheCommands::Pack { node_fp, out, cache_dir } => {
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
                let _dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
                let report = verify_cache_dirs(&dir, remote.as_ref().map(|v| v.as_path()))?;
                let corrupt = report["corrupt_total"].as_u64().unwrap_or(0);
                if cli.json {
                    return emit_json(
                        &cli,
                        "dag.cache.verify",
                        corrupt == 0,
                        report,
                        Vec::new(),
                        if corrupt == 0 { ExitCode::SUCCESS } else { ExitCode::from(3) },
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
            CacheCommands::Diff { cache_dir, key_a, key_b } => {
                let dir = cache_dir.clone().or_else(env_cache_dir).ok_or(ExitCode::from(3))?;
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
        Commands::Adapters { command } => {
            routes::adapter_routes::handle_adapters_command(&cli, command)
        }
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
        Commands::EquivalenceProof { run_a, run_b, backend_a, backend_b } => {
            routes::surface_routes::handle_equivalence_proof_command(
                &cli, run_a, run_b, backend_a, backend_b,
            )
        }
        Commands::VersionInspect { dag, run_dir, export_bundle } => {
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
            ConfigCommands::ShowEffective { config, jobs, cache_mode, materialize_inputs } => {
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

fn emit_command_access_denial(
    cli: &DagCli,
    denial: CommandAccessDenial,
) -> Result<ExitCode, ExitCode> {
    let command = format!("dag.{}", denial.root_command);
    let lane = lane_label(denial.lane);
    let message = denial.message();
    let hint = format!(
        "set {}=1 to run this {} route intentionally or use `bijux-dag commands --lane {}` to inspect this non-stable access lane",
        denial.opt_in_env,
        lane,
        lane
    );
    if cli.json {
        return emit_json(
            cli,
            &command,
            false,
            json!({
                "command_family": denial.root_command,
                "lane": denial.lane,
                "access": "opt-in",
                "opt_in_env": denial.opt_in_env,
            }),
            vec![json!({
                "code": "release-boundary-opt-in",
                "message": message,
                "hint": hint,
            })],
            ExitCode::from(2),
        );
    }
    eprintln!("{message}");
    eprintln!("{hint}");
    Err(ExitCode::from(2))
}

pub(crate) fn read_file(path: &Path) -> Result<String, ExitCode> {
    fs_input::read_utf8_file(path).map_err(|_| ExitCode::from(3))
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

pub(crate) fn selector_cli_string(selector: &bijux_dag_runtime::Selector) -> String {
    match selector {
        bijux_dag_runtime::Selector::Id(v) => format!("id:{v}"),
        bijux_dag_runtime::Selector::IdPrefix(v) => format!("id-prefix:{v}"),
        bijux_dag_runtime::Selector::Tag(v) => format!("tag:{v}"),
        bijux_dag_runtime::Selector::Kind(v) => format!("kind:{v}"),
    }
}

pub(crate) fn parse_graph(input: &str) -> Result<Graph, ExitCode> {
    match read_graph::parse_graph_with_compat(input) {
        Ok(g) => Ok(g),
        Err(GraphError::Json(_)) => Err(ExitCode::from(2)),
        Err(GraphError::InvalidSpec(_)) => Err(ExitCode::from(1)),
        Err(_) => Err(ExitCode::from(3)),
    }
}

pub(crate) fn load_graphs_or_emit(
    cli: &commands::DagCli,
    command_name: &str,
    dags: &[PathBuf],
) -> Result<Graph, ExitCode> {
    match read_graph::load_graphs(dags) {
        Ok(graph) => Ok(graph),
        Err(error) => {
            let code = error.exit_code();
            if cli.json {
                let _ = emit_json(
                    cli,
                    command_name,
                    false,
                    json!({
                        "error": error.to_string(),
                        "dags": dags,
                    }),
                    Vec::new(),
                    code,
                );
            } else if !cli.quiet {
                eprintln!("{error}");
            }
            Err(code)
        }
    }
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

fn collect_json_diff_paths(path: &str, left: &Value, right: &Value, out: &mut Vec<String>) {
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys = std::collections::BTreeSet::new();
            keys.extend(a.keys().cloned());
            keys.extend(b.keys().cloned());
            for key in keys {
                let child =
                    if path.is_empty() { format!("/{}", key) } else { format!("{}/{}", path, key) };
                match (a.get(&key), b.get(&key)) {
                    (Some(lv), Some(rv)) => collect_json_diff_paths(&child, lv, rv, out),
                    _ => out.push(child),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            let max = a.len().max(b.len());
            for idx in 0..max {
                let child =
                    if path.is_empty() { format!("/{}", idx) } else { format!("{}/{}", path, idx) };
                match (a.get(idx), b.get(idx)) {
                    (Some(lv), Some(rv)) => collect_json_diff_paths(&child, lv, rv, out),
                    _ => out.push(child),
                }
            }
        }
        _ => {
            if left != right {
                out.push(if path.is_empty() { "/".to_string() } else { path.to_string() });
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
    if bundle.get("node_traces").and_then(|v| v.as_object()).is_none() {
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
        if !bundle.get("outputs").is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty())) {
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
        if !bundle.get("node_traces").is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty())) {
            violations.push(
                "INV-EXPORT-MODE-001 provenance-only bundle must include empty node_traces map"
                    .to_string(),
            );
        }
        if !bundle.get("outputs").is_some_and(|v| v.as_object().is_some_and(|m| m.is_empty())) {
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
    let status = report.get("status").and_then(Value::as_str).unwrap_or("error").to_string();
    let errors = report.get("errors").and_then(Value::as_array).cloned().unwrap_or_default();
    let invariant_violations =
        report.get("invariant_violations").and_then(Value::as_array).cloned().unwrap_or_default();
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
        value.get("source").and_then(Value::as_str).unwrap_or("native-run").to_string()
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
        assert!(violations.iter().any(|v| v.contains("INV-EXPORT-VERIFY-001")));
        assert!(violations.iter().any(|v| v.contains("INV-TRACE-ATTEMPT-001")));
    }
}

#[cfg(test)]
mod cache_archive_hardening_tests {
    use super::ExitCode;
    use crate::cache::unpack_cache_archive_bounded;
    use tar::{Builder, Header};

    fn unpack_status(bytes: Vec<u8>) -> Result<(), ExitCode> {
        let dec = flate2::read::GzDecoder::new(std::io::Cursor::new(bytes));
        let mut archive = tar::Archive::new(dec);
        let dst = tempfile::tempdir().expect("tempdir");
        unpack_cache_archive_bounded(&mut archive, dst.path())
    }

    #[test]
    fn cache_unpack_rejects_oversized_archive_entries() {
        let mut tar_bytes = Vec::new();
        {
            let enc = flate2::write::GzEncoder::new(&mut tar_bytes, flate2::Compression::default());
            let mut builder = Builder::new(enc);
            let mut header = Header::new_gnu();
            let payload = vec![0u8; (9 * 1024 * 1024) as usize];
            header.set_size(payload.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, "meta.json", payload.as_slice()).expect("append");
            let enc = builder.into_inner().expect("encoder");
            enc.finish().expect("finish");
        }
        assert!(unpack_status(tar_bytes).is_err());
    }

    #[test]
    fn cache_unpack_rejects_symlink_entries() {
        let mut tar_bytes = Vec::new();
        {
            let enc = flate2::write::GzEncoder::new(&mut tar_bytes, flate2::Compression::default());
            let mut builder = Builder::new(enc);
            let mut header = Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_link(&mut header, "bad-link", "/tmp/escape").expect("append symlink");
            let enc = builder.into_inner().expect("encoder");
            enc.finish().expect("finish");
        }
        assert!(unpack_status(tar_bytes).is_err());
    }

    #[test]
    fn cache_unpack_accepts_regular_files_and_directories() {
        let mut tar_bytes = Vec::new();
        {
            let enc = flate2::write::GzEncoder::new(&mut tar_bytes, flate2::Compression::default());
            let mut builder = Builder::new(enc);

            let mut dir_header = Header::new_gnu();
            dir_header.set_entry_type(tar::EntryType::Directory);
            dir_header.set_size(0);
            dir_header.set_mode(0o755);
            dir_header.set_cksum();
            builder.append_data(&mut dir_header, "node", std::io::empty()).expect("dir");

            let mut file_header = Header::new_gnu();
            let body = br#"{"node_fingerprint":"k"}"#;
            file_header.set_size(body.len() as u64);
            file_header.set_mode(0o644);
            file_header.set_cksum();
            builder.append_data(&mut file_header, "meta.json", &body[..]).expect("file");

            let enc = builder.into_inner().expect("encoder");
            enc.finish().expect("finish");
        }
        let status = unpack_status(tar_bytes);
        assert!(status.is_ok());
    }
}
