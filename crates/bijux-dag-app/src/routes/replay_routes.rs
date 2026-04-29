use crate::commands::{CacheModeArg, DagCli, MaterializeModeArg};
use crate::graph_helpers::parse_selectors;
use crate::replay_cmd::ReplayCommandResponse;
use crate::run_data::{load_snapshot, map_materialize_mode};
use crate::{
    build_run_proof_bundle, emit_json, read_run_id, selector_cli_string, CacheMode, ExitCode,
    Runtime, RuntimeConfig, Value,
};
use serde_json::json;
use std::path::Path;
use std::path::PathBuf;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_replay_command(
    cli: &DagCli,
    run_dir: &Path,
    out: &Path,
    dry_run: bool,
    sandbox: bool,
    prove: bool,
    reuse_cache: bool,
    cache: CacheModeArg,
    jobs: usize,
    run_id: Option<String>,
    cpu_budget: Option<u32>,
    deny_network: bool,
    deny_env: bool,
    deny_clock: bool,
    clean_env: bool,
    hermetic: bool,
    select: &[String],
    exclude: &[String],
    materialize_inputs: MaterializeModeArg,
    remote_cache_dir: Option<PathBuf>,
) -> Result<ExitCode, ExitCode> {
    let snapshot = load_snapshot(run_dir)?;
    let source_run_id = read_run_id(run_dir).ok();
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
    let selectors = parse_selectors(select, exclude)?;
    if dry_run {
        let dry_select = selectors.include.iter().map(selector_cli_string).collect::<Vec<_>>();
        let dry_exclude = selectors.exclude.iter().map(selector_cli_string).collect::<Vec<_>>();
        let plan = crate::replay_service::replay_dry_run_plan(
            run_dir,
            out,
            &snapshot,
            source_run_id.as_deref(),
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
    if sandbox && out.starts_with(run_dir) {
        return Err(ExitCode::from(3));
    }
    let options = RuntimeConfig {
        jobs,
        cpu_budget,
        run_timeout_ms: None,
        node_timeout_ms: None,
        materialize_inputs: map_materialize_mode(materialize_inputs),
        cache_mode,
        cache_dir: None,
        remote_cache_dir,
        run_id,
        parent_run_id: source_run_id,
        latest_symlink: None,
        policy: bijux_dag_runtime::PolicyConfig {
            deny_network: deny_network_effective,
            deny_env: deny_env_effective,
            deny_clock: deny_clock_effective,
            clean_env: clean_env_effective,
        },
        selectors,
        ..RuntimeConfig::default()
    };
    let run_path = runtime.run(&snapshot.graph, out, options).map_err(|_| ExitCode::from(3))?;
    let replay_proof = if prove {
        let diff = crate::replay_service::run_diff_from_dirs(run_dir, &run_path)?;
        let source_evidence_gaps = crate::replay_service::replay_evidence_gaps(run_dir);
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
            "source_run_id": read_run_id(run_dir)?,
            "replay_run_id": read_run_id(&run_path)?,
            "sandbox_mode": if sandbox { "isolated" } else { "standard" }
        }))
    } else {
        None
    };
    let response =
        ReplayCommandResponse { run_dir: Some(run_path.clone()), dry_run_plan: None, replay_proof };
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

pub(crate) fn handle_prove_command(cli: &DagCli, run_dir: &Path) -> Result<ExitCode, ExitCode> {
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
