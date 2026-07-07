use crate::commands::{CacheModeArg, DagCli, MaterializeModeArg};
use crate::graph_helpers::{
    parse_selectors, resolve_downstream_run_selection, validate_downstream_selection_surface,
};
use crate::replay_cmd::ReplayCommandResponse;
use crate::routes::policy_surface::{
    cache_surface_payload, policy_surface_payload, replay_sandbox_scope_payload,
};
use crate::routes::preconditions::{require_run_directory, require_safe_path};
use crate::routes::resource_capacity_args::parse_resource_capacities;
use crate::run_data::{load_snapshot, map_materialize_mode};
use crate::{
    build_run_proof_bundle, emit_json, read_run_id, resolve_run_dir, selector_cli_string,
    CacheMode, ExitCode, Runtime, RuntimeConfig, Value,
};
use serde_json::json;
use std::path::Path;
use std::path::PathBuf;

fn build_replay_runtime_options(
    jobs: usize,
    cpu_budget: Option<u32>,
    memory_budget_mb: Option<u32>,
    gpu_device_budget: Option<u32>,
    named_resource_capacities: std::collections::BTreeMap<String, u32>,
    run_id: Option<String>,
    source_run_id: Option<String>,
    source_run_dir: Option<PathBuf>,
    cache_mode: CacheMode,
    remote_cache_dir: Option<PathBuf>,
    downstream_selection_roots: Vec<String>,
    selectors: bijux_dag_runtime::SelectorSet,
    dependency_closure: bool,
    materialize_inputs: MaterializeModeArg,
    policy: bijux_dag_runtime::PolicyConfig,
) -> RuntimeConfig {
    RuntimeConfig {
        jobs,
        cpu_budget,
        memory_budget_mb,
        gpu_device_budget,
        named_resource_capacities,
        run_timeout_ms: None,
        node_timeout_ms: None,
        materialize_inputs: map_materialize_mode(materialize_inputs),
        cache_mode,
        cache_dir: None,
        remote_cache_dir,
        run_id,
        parent_run_id: source_run_id,
        replay_source_run_dir: source_run_dir,
        latest_symlink: None,
        policy,
        downstream_selection_roots,
        selectors,
        partial_rerun_dependency_closure: dependency_closure,
        scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
            max_parallelism: jobs.max(1),
            ..bijux_dag_runtime::SchedulerPolicy::default()
        },
        ..RuntimeConfig::default()
    }
}

fn resolve_replay_source_run(
    run_dir: Option<&Path>,
    source_run_id: Option<&str>,
    source_run_root: Option<&Path>,
    out: &Path,
) -> Result<PathBuf, ExitCode> {
    match (run_dir, source_run_id) {
        (Some(run_dir), None) => {
            require_run_directory(run_dir)?;
            Ok(run_dir.to_path_buf())
        }
        (None, Some(source_run_id)) => {
            let root = source_run_root.unwrap_or(out);
            let resolved = resolve_run_dir(root, source_run_id);
            require_run_directory(&resolved)?;
            Ok(resolved)
        }
        _ => Err(ExitCode::from(2)),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_replay_command(
    cli: &DagCli,
    run_dir: Option<&Path>,
    source_run_id: Option<&str>,
    source_run_root: Option<&Path>,
    out: &Path,
    dry_run: bool,
    sandbox: bool,
    prove: bool,
    reuse_cache: bool,
    cache: CacheModeArg,
    jobs: usize,
    run_id: Option<String>,
    cpu_budget: Option<u32>,
    memory_budget_mb: Option<u32>,
    gpu_device_budget: Option<u32>,
    resource_capacity: &[String],
    deny_network: bool,
    deny_env: bool,
    deny_clock: bool,
    clean_env: bool,
    hermetic: bool,
    from_node: &[String],
    select: &[String],
    exclude: &[String],
    dependency_closure: bool,
    materialize_inputs: MaterializeModeArg,
    remote_cache_dir: Option<PathBuf>,
) -> Result<ExitCode, ExitCode> {
    let run_dir = resolve_replay_source_run(run_dir, source_run_id, source_run_root, out)?;
    require_safe_path(out)?;
    let snapshot = load_snapshot(&run_dir)?;
    let source_run_id = read_run_id(&run_dir).ok();
    let runtime = Runtime::new();
    let named_resource_capacities = parse_resource_capacities(resource_capacity)?;
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
    let mut deny_network_effective = deny_network;
    let mut deny_clock_effective = deny_clock;
    let deny_env_effective = deny_env;
    let mut clean_env_effective = clean_env;
    if !clean_env_effective {
        clean_env_effective = true;
    }
    if hermetic {
        deny_network_effective = true;
        deny_clock_effective = true;
        clean_env_effective = true;
    }
    validate_downstream_selection_surface(from_node, select, exclude, dependency_closure)?;
    let (downstream_selection_roots, downstream_selected_nodes) =
        resolve_downstream_run_selection(&snapshot.graph, from_node)?;
    let boundary_verification = if downstream_selection_roots.is_empty() {
        None
    } else {
        let source_run_id = source_run_id.as_deref().ok_or(ExitCode::from(3))?;
        let report = crate::replay_service::verify_replay_boundary_inputs(
            &run_dir,
            source_run_id,
            &downstream_selection_roots,
        )?;
        if !report.verified {
            if cli.json {
                return emit_json(
                    cli,
                    "dag.replay",
                    false,
                    json!({
                        "message": "upstream artifact verification failed for the requested replay boundary",
                        "upstream_artifact_verification": report,
                    }),
                    Vec::new(),
                    ExitCode::from(3),
                );
            }
            eprintln!("upstream artifact verification failed for the requested replay boundary");
            for error in &report.errors {
                eprintln!("error: {error}");
            }
            for check in &report.checks {
                for note in &check.notes {
                    eprintln!(
                        "input {} <- {}:{}: {}",
                        check.boundary_node_id,
                        check.source_node_id,
                        check.source_output_name,
                        note
                    );
                }
            }
            return Err(ExitCode::from(3));
        }
        Some(report)
    };
    let selectors = if downstream_selection_roots.is_empty() {
        parse_selectors(select, exclude)?
    } else {
        bijux_dag_runtime::SelectorSet::default()
    };
    let rerun_node_id =
        (downstream_selection_roots.len() == 1).then(|| downstream_selection_roots[0].clone());
    let options = build_replay_runtime_options(
        jobs,
        cpu_budget,
        memory_budget_mb,
        gpu_device_budget,
        named_resource_capacities,
        run_id,
        source_run_id.clone(),
        Some(run_dir.clone()),
        cache_mode.clone(),
        remote_cache_dir,
        downstream_selection_roots.clone(),
        selectors.clone(),
        dependency_closure,
        materialize_inputs,
        bijux_dag_runtime::PolicyConfig {
            deny_network: deny_network_effective,
            deny_env: deny_env_effective,
            deny_clock: deny_clock_effective,
            clean_env: clean_env_effective,
            ..bijux_dag_runtime::PolicyConfig::default()
        },
    );
    if dry_run {
        let dry_select = selectors.include.iter().map(selector_cli_string).collect::<Vec<_>>();
        let dry_exclude = selectors.exclude.iter().map(selector_cli_string).collect::<Vec<_>>();
        let plan = crate::replay_service::replay_dry_run_plan(
            &run_dir,
            out,
            &snapshot,
            source_run_id.as_deref(),
            &downstream_selection_roots,
            &downstream_selected_nodes,
            &dry_select,
            &dry_exclude,
            &format!("{cache_mode:?}"),
            jobs,
            prove,
            sandbox,
        )?;
        let response = ReplayCommandResponse {
            run_dir: None,
            dry_run_plan: Some(plan.clone()),
            replay_proof: None,
            upstream_artifact_verification: boundary_verification.map(|report| {
                serde_json::to_value(report).expect("boundary verification should serialize")
            }),
            node_rerun_diff: None,
            cache_surface: Some(cache_surface_payload(&options)),
            policy_surface: Some(policy_surface_payload(&snapshot.graph, &options, hermetic)?),
            sandbox_scope: Some(replay_sandbox_scope_payload(sandbox)),
        };
        if cli.json {
            return emit_json(
                cli,
                "dag.replay",
                true,
                serde_json::to_value(&response).map_err(|_| ExitCode::from(3))?,
                Vec::new(),
                ExitCode::SUCCESS,
            );
        }
        println!("{}", serde_json::to_string_pretty(&plan).unwrap());
        return Ok(ExitCode::SUCCESS);
    }
    if sandbox && out.starts_with(&run_dir) {
        return Err(ExitCode::from(3));
    }
    let run_path =
        runtime.run(&snapshot.graph, out, options.clone()).map_err(|_| ExitCode::from(3))?;
    let replay_proof = if prove {
        let diff = crate::replay_service::run_diff_from_dirs(&run_dir, &run_path)?;
        let source_evidence_gaps = crate::replay_service::replay_evidence_gaps(&run_dir);
        let replay_evidence_gaps = crate::replay_service::replay_evidence_gaps(&run_path);
        let safety_level = if !source_evidence_gaps.is_empty() || !replay_evidence_gaps.is_empty() {
            "incomplete_evidence".to_string()
        } else {
            serde_json::to_value(diff.replay_equivalence.safety_level)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "unsupported".to_string())
        };
        Some(json!({
            "fidelity_level": if diff.replay_equivalence.equivalent { "strict_equivalent" } else { "diverged" },
            "equivalent": diff.replay_equivalence.equivalent,
            "safety_level": safety_level,
            "reasons": diff.replay_equivalence.reasons,
            "reason_report": diff.replay_equivalence.reason_report,
            "cause_groups": diff.replay_equivalence.cause_groups,
            "branch_decision_drift_nodes": diff.replay_equivalence.branch_decision_drift_nodes,
            "source_evidence_gaps": source_evidence_gaps,
            "replay_evidence_gaps": replay_evidence_gaps,
            "source_run_id": read_run_id(&run_dir)?,
            "replay_run_id": read_run_id(&run_path)?,
            "sandbox_mode": if sandbox { "isolated" } else { "standard" }
        }))
    } else {
        None
    };
    let node_rerun_diff = rerun_node_id
        .as_deref()
        .map(|node_id| crate::replay_service::node_rerun_diff_report(&run_dir, &run_path, node_id))
        .transpose()?;
    let response = ReplayCommandResponse {
        run_dir: Some(run_path.clone()),
        dry_run_plan: None,
        replay_proof,
        upstream_artifact_verification: boundary_verification.map(|report| {
            serde_json::to_value(report).expect("boundary verification should serialize")
        }),
        node_rerun_diff,
        cache_surface: Some(cache_surface_payload(&options)),
        policy_surface: Some(policy_surface_payload(&snapshot.graph, &options, hermetic)?),
        sandbox_scope: Some(replay_sandbox_scope_payload(sandbox)),
    };
    if cli.json {
        return emit_json(
            cli,
            "dag.replay",
            true,
            serde_json::to_value(&response).map_err(|_| ExitCode::from(3))?,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    if !cli.quiet {
        println!("run dir: {}", run_path.display());
        if let Some(diff) = &response.node_rerun_diff {
            println!(
                "rerun_node: {}",
                diff.get("node_id").and_then(Value::as_str).unwrap_or("unknown")
            );
            println!(
                "rerun_node_equivalent: {}",
                diff.get("summary")
                    .and_then(|summary| summary.get("equivalent"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            );
        }
        if let Some(proof) = &response.replay_proof {
            println!(
                "replay_proof_fidelity: {}",
                proof.get("fidelity_level").and_then(Value::as_str).unwrap_or("unknown")
            );
            if let Some(reasons) = proof.get("reasons").and_then(Value::as_array) {
                if !reasons.is_empty() {
                    println!(
                        "replay_proof_reasons: {}",
                        reasons.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", ")
                    );
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{build_replay_runtime_options, resolve_replay_source_run};
    use crate::commands::MaterializeModeArg;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn replay_runtime_options_preserve_selector_and_closure_configuration() {
        let selectors = bijux_dag_runtime::SelectorSet {
            include: vec![bijux_dag_runtime::Selector::Id("report".to_string())],
            exclude: vec![bijux_dag_runtime::Selector::Kind("const".to_string())],
        };
        let options = build_replay_runtime_options(
            2,
            Some(8),
            Some(2048),
            Some(2),
            std::collections::BTreeMap::from([
                ("database_slot".to_string(), 2),
                ("license.render".to_string(), 1),
            ]),
            Some("replay-run".to_string()),
            Some("source-run".to_string()),
            Some(PathBuf::from("/tmp/source-run")),
            crate::CacheMode::ReadWrite,
            Some(PathBuf::from("/tmp/remote-cache")),
            vec!["transform".to_string()],
            selectors.clone(),
            true,
            MaterializeModeArg::Symlink,
            bijux_dag_runtime::PolicyConfig {
                deny_network: true,
                deny_env: false,
                deny_clock: true,
                clean_env: true,
                ..bijux_dag_runtime::PolicyConfig::default()
            },
        );

        assert_eq!(options.jobs, 2);
        assert_eq!(options.scheduler_policy.max_parallelism, 2);
        assert_eq!(options.cpu_budget, Some(8));
        assert_eq!(options.memory_budget_mb, Some(2048));
        assert_eq!(options.replay_source_run_dir, Some(PathBuf::from("/tmp/source-run")));
        assert_eq!(options.gpu_device_budget, Some(2));
        assert_eq!(options.named_resource_capacities.get("database_slot"), Some(&2));
        assert_eq!(options.named_resource_capacities.get("license.render"), Some(&1));
        assert_eq!(options.run_id.as_deref(), Some("replay-run"));
        assert_eq!(options.parent_run_id.as_deref(), Some("source-run"));
        assert!(options.partial_rerun_dependency_closure);
        assert_eq!(options.downstream_selection_roots, vec!["transform".to_string()]);
        assert_eq!(options.selectors.include.len(), selectors.include.len());
        assert_eq!(options.selectors.exclude.len(), selectors.exclude.len());
        assert!(matches!(options.materialize_inputs, bijux_dag_runtime::MaterializeMode::Symlink));
        assert!(matches!(options.cache_mode, crate::CacheMode::ReadWrite));
        assert!(options.policy.deny_network);
        assert!(options.policy.deny_clock);
    }

    #[test]
    fn resolve_replay_source_run_accepts_explicit_run_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_dir = tmp.path().join("run-explicit");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(run_dir.join("manifest.json"), r#"{"run_id":"explicit"}"#)
            .expect("write manifest");

        let resolved = resolve_replay_source_run(Some(&run_dir), None, None, tmp.path())
            .expect("resolve run dir");
        assert_eq!(resolved, run_dir);
    }

    #[test]
    fn resolve_replay_source_run_resolves_run_id_against_source_root() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let run_root = tmp.path().join("runs");
        let run_dir = run_root.join("run-source-123");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(run_dir.join("manifest.json"), r#"{"run_id":"source-123"}"#)
            .expect("write manifest");

        let resolved = resolve_replay_source_run(
            None,
            Some("source-123"),
            Some(&run_root),
            Path::new("/unused/out"),
        )
        .expect("resolve source run id");
        assert_eq!(resolved, run_dir);
    }
}

pub(crate) fn handle_prove_command(cli: &DagCli, run_dir: &Path) -> Result<ExitCode, ExitCode> {
    require_run_directory(run_dir)?;
    let proof = build_run_proof_bundle(run_dir)?;
    let complete = proof.get("complete").and_then(Value::as_bool).unwrap_or(false);
    if cli.json {
        return emit_json(
            cli,
            "dag.prove",
            complete,
            proof,
            Vec::new(),
            if complete { ExitCode::SUCCESS } else { ExitCode::from(3) },
        );
    }
    println!("proof id: {}", proof["proof_id"]);
    println!("status: {}", proof["status"]);
    println!("complete: {}", complete);
    println!("determinism: {}", proof["determinism"]);
    if let Some(reasons) = proof["incomplete_reasons"].as_array() {
        if !reasons.is_empty() {
            println!("incomplete_reasons: {}", proof["incomplete_reasons"]);
        }
    }
    if complete {
        Ok(ExitCode::SUCCESS)
    } else {
        Err(ExitCode::from(3))
    }
}

pub(crate) fn handle_proof_summary_command(
    cli: &DagCli,
    run_dir: &Path,
) -> Result<ExitCode, ExitCode> {
    require_run_directory(run_dir)?;
    let proof = build_run_proof_bundle(run_dir)?;
    let complete = proof.get("complete").and_then(Value::as_bool).unwrap_or(false);
    let status = proof.get("status").and_then(Value::as_str).unwrap_or("incomplete");
    let determinism =
        proof.get("determinism").and_then(Value::as_str).unwrap_or("insufficient-evidence");
    let integrity =
        proof.get("integrity").and_then(Value::as_str).unwrap_or("insufficient-evidence");
    let replay_level = proof
        .get("replay_evidence")
        .and_then(|v| v.get("level"))
        .and_then(Value::as_str)
        .unwrap_or("partial");
    let incomplete_reasons =
        proof.get("incomplete_reasons").and_then(Value::as_array).cloned().unwrap_or_default();
    let summary = json!({
        "proof_id": proof.get("proof_id").cloned().unwrap_or(Value::Null),
        "run_id": proof.get("run_id").cloned().unwrap_or(Value::Null),
        "status": status,
        "complete": complete,
        "determinism": determinism,
        "integrity": integrity,
        "replay_level": replay_level,
        "incomplete_reasons": incomplete_reasons
    });
    if cli.json {
        return emit_json(
            cli,
            "dag.proof-summary",
            complete,
            summary,
            Vec::new(),
            if complete { ExitCode::SUCCESS } else { ExitCode::from(3) },
        );
    }

    println!("proof summary: {}", summary);
    if complete {
        Ok(ExitCode::SUCCESS)
    } else {
        Err(ExitCode::from(3))
    }
}
