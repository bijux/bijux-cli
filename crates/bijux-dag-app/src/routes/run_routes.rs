use crate::commands::{AbsolutePathPolicyArg, CacheModeArg, DagCli, MaterializeModeArg};
use crate::routes::plan_routes::{
    concise_plan_lines, plan_explain_payload, resolve_plan_preview_layout,
};
use crate::routes::policy_surface::policy_surface_payload;
use crate::routes::preconditions::{require_file, require_safe_path};
use crate::run_data::map_materialize_mode;
use crate::runtime_inputs::{bind_runtime_inputs, missing_required_graph_inputs};
use crate::{emit_json, load_graphs_or_emit, parse_selectors, ExitCode};
use bijux_dag_runtime::{
    build_planner_analysis, registered_adapters, CacheMode, PlannerGuardrails, Runtime,
    RuntimeConfig,
};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct RunRouteRequest<'a> {
    pub dags: &'a [PathBuf],
    pub out: &'a Path,
    pub input: &'a Vec<String>,
    pub inputs_file: Option<PathBuf>,
    pub run_id: Option<String>,
    pub latest: Option<PathBuf>,
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub node_timeout_ms: Option<u64>,
    pub run_timeout_ms: Option<u64>,
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    pub hermetic: bool,
    pub select: &'a Vec<String>,
    pub exclude: &'a Vec<String>,
    pub materialize_inputs: MaterializeModeArg,
    pub cache: CacheModeArg,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicyArg,
    pub preflight_only: bool,
    pub explain_scheduling: bool,
}

fn cache_preflight(cache_mode: CacheModeArg, cache_dir: &Option<PathBuf>) -> serde_json::Value {
    if matches!(cache_mode, CacheModeArg::Off) {
        return json!({"status":"disabled"});
    }
    let Some(dir) = cache_dir.as_ref() else {
        return json!({"status":"implicit"});
    };
    if fs::create_dir_all(dir).is_err() {
        return json!({"status":"error","path":dir,"writable":false});
    }
    let probe = dir.join(".__bijux_preflight_probe");
    let writable = fs::write(&probe, b"ok").is_ok();
    let _ = fs::remove_file(&probe);
    json!({"status": if writable { "ok" } else { "error" }, "path": dir, "writable": writable})
}

pub(crate) fn handle_run_command(
    cli: &DagCli,
    req: RunRouteRequest<'_>,
) -> Result<ExitCode, ExitCode> {
    for dag in req.dags {
        require_file(dag)?;
    }
    require_safe_path(req.out)?;
    let mut graph = load_graphs_or_emit(cli, "dag.run", req.dags)?;
    let runtime_inputs =
        match bind_runtime_inputs(&graph.inputs, req.inputs_file.as_deref(), req.input) {
            Ok(binding) => binding,
            Err(message) => {
                return emit_run_input_error(cli, &message, json!({ "error": message }));
            }
        };
    graph.inputs = runtime_inputs.bound_inputs.clone();
    let missing_inputs = missing_required_graph_inputs(&graph);
    if !missing_inputs.is_empty() {
        let message = format!("missing required runtime inputs: {}", missing_inputs.join(", "));
        return emit_run_input_error(
            cli,
            &message,
            json!({
                "error": message,
                "missing_inputs": missing_inputs,
            }),
        );
    }
    let runtime = Runtime::new();
    let (deny_network, deny_clock, clean_env) =
        effective_policy_flags(req.deny_network, req.deny_clock, req.clean_env, req.hermetic);
    let deny_env = req.deny_env;
    let selectors = parse_selectors(req.select, req.exclude)?;
    let cache_dir = req.cache_dir.clone();
    let remote_cache_dir = req.remote_cache_dir.clone();
    let preview_layout = resolve_plan_preview_layout(Some(req.out), req.run_id.as_deref())?;
    let absolute_path_policy = req.absolute_path_policy.into();
    let options = RuntimeConfig {
        jobs: req.jobs,
        cpu_budget: req.cpu_budget,
        run_timeout_ms: req.run_timeout_ms,
        node_timeout_ms: req.node_timeout_ms,
        materialize_inputs: map_materialize_mode(req.materialize_inputs),
        cache_mode: match req.cache {
            CacheModeArg::Off => CacheMode::Off,
            CacheModeArg::Read => CacheMode::Read,
            CacheModeArg::Readwrite => CacheMode::ReadWrite,
        },
        cache_dir: cache_dir.clone(),
        remote_cache_dir,
        run_root: Some(req.out.to_path_buf()),
        absolute_path_policy,
        run_id: preview_layout.as_ref().map(|layout| layout.run_id.clone()),
        latest_symlink: req.latest,
        policy: bijux_dag_runtime::PolicyConfig { deny_network, deny_env, deny_clock, clean_env },
        selectors,
        ..RuntimeConfig::default()
    };
    let scheduling = if req.preflight_only || req.explain_scheduling {
        Some(
            build_planner_analysis(
                &graph,
                &options,
                &options.selectors,
                &PlannerGuardrails { allow_semantic_optimizations: true },
            )
            .map_err(|_| ExitCode::from(3))?,
        )
    } else {
        None
    };
    if req.preflight_only {
        let payload = json!({
            "dags": req.dags,
            "adapters": registered_adapters(),
            "cache": cache_preflight(req.cache, &cache_dir),
            "run_layout": preview_layout,
            "policy": {
                "deny_network": options.policy.deny_network,
                "deny_env": options.policy.deny_env,
                "deny_clock": options.policy.deny_clock,
                "clean_env": options.policy.clean_env,
            },
            "policy_surface": policy_surface_payload(&graph, &options, req.hermetic)?,
            "input_summary": runtime_inputs.human_summary,
            "redacted_input_keys": runtime_inputs.redacted_keys,
            "selectors": {
                "include": req.select,
                "exclude": req.exclude,
            },
            "scheduling": scheduling
                .as_ref()
                .map(|result| {
                    plan_explain_payload(
                        result,
                        preview_layout.as_ref(),
                        absolute_path_policy,
                    )
                }),
        });
        if cli.json {
            return emit_json(
                cli,
                "dag.run.preflight",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            );
        }
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        return Ok(ExitCode::SUCCESS);
    }
    let run_path = runtime.run(&graph, req.out, options).map_err(|_| ExitCode::from(3))?;

    if cli.json {
        return emit_json(
            cli,
            "dag.run",
            true,
            json!({
                "run_dir": run_path,
                "run_layout": preview_layout,
                "scheduling": scheduling
                    .as_ref()
                    .map(|result| {
                        plan_explain_payload(
                            result,
                            preview_layout.as_ref(),
                            absolute_path_policy,
                        )
                    }),
            }),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    if let Some(scheduling) = scheduling.as_ref() {
        for line in concise_plan_lines(scheduling) {
            println!("{line}");
        }
    }
    if !cli.quiet {
        if !runtime_inputs.human_summary.is_empty() {
            println!("inputs: {}", serde_json::to_string(&runtime_inputs.human_summary).unwrap());
        }
        if !runtime_inputs.redacted_keys.is_empty() {
            println!("redacted_inputs: {:?}", runtime_inputs.redacted_keys);
        }
        println!("run dir: {}", run_path.display());
    }
    Ok(ExitCode::SUCCESS)
}

fn emit_run_input_error(
    cli: &DagCli,
    message: &str,
    payload: serde_json::Value,
) -> Result<ExitCode, ExitCode> {
    if cli.json {
        return emit_json(cli, "dag.run", false, payload, Vec::new(), ExitCode::from(2));
    }
    eprintln!("{message}");
    Err(ExitCode::from(2))
}

fn effective_policy_flags(
    deny_network: bool,
    deny_clock: bool,
    clean_env: bool,
    hermetic: bool,
) -> (bool, bool, bool) {
    if hermetic {
        return (true, true, true);
    }
    let _ = clean_env;
    let normalized_clean_env = true;
    (deny_network, deny_clock, normalized_clean_env)
}

#[cfg(test)]
mod tests {
    use super::{
        cache_preflight, effective_policy_flags, emit_run_input_error, handle_run_command,
        RunRouteRequest,
    };
    use crate::commands::{
        AbsolutePathPolicyArg, CacheModeArg, Commands, DagCli, MaterializeModeArg,
    };
    use crate::ExitCode;
    use serde_json::json;
    use std::fs;

    #[test]
    fn hermetic_forces_isolation_flags() {
        assert_eq!(effective_policy_flags(false, false, false, true), (true, true, true));
    }

    #[test]
    fn non_hermetic_preserves_network_clock_and_normalizes_clean_env() {
        assert_eq!(effective_policy_flags(true, false, false, false), (true, false, true));
    }

    #[test]
    fn cache_preflight_reports_disabled_when_cache_is_off() {
        assert_eq!(cache_preflight(CacheModeArg::Off, &None)["status"], "disabled");
    }

    #[test]
    fn run_input_error_returns_cli_error_code() {
        let cli = DagCli { json: false, quiet: true, command: Commands::Version };
        let code = emit_run_input_error(&cli, "missing required runtime inputs: region", json!({}))
            .expect_err("error code");
        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn run_preflight_accepts_composed_graph_fragments() {
        let dir = tempfile::tempdir().expect("tmp");
        let foundation = dir.path().join("foundation.json");
        let publication = dir.path().join("publication.json");
        let out = dir.path().join("runs");
        fs::write(
            &foundation,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"extract","kind":"const","outputs":[{"name":"report","path":"extract/report.json"}],"params":{"value":"seed"}}],
              "edges":[]
            }"#,
        )
        .expect("write foundation");
        fs::write(
            &publication,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[{"id":"publish","kind":"const","inputs":["report"],"outputs":[{"name":"out","path":"publish/out.json"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}}],
              "edges":[{"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}]
            }"#,
        )
        .expect("write publication");

        let cli = DagCli { json: true, quiet: true, command: Commands::Version };
        let code = handle_run_command(
            &cli,
            RunRouteRequest {
                dags: &[foundation, publication],
                out: &out,
                input: &Vec::new(),
                inputs_file: None,
                run_id: Some("previewed".to_string()),
                latest: None,
                jobs: 1,
                cpu_budget: None,
                node_timeout_ms: None,
                run_timeout_ms: None,
                deny_network: false,
                deny_env: false,
                deny_clock: false,
                clean_env: false,
                hermetic: false,
                select: &Vec::new(),
                exclude: &Vec::new(),
                materialize_inputs: MaterializeModeArg::Copy,
                cache: CacheModeArg::Off,
                cache_dir: None,
                remote_cache_dir: None,
                absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
                preflight_only: true,
                explain_scheduling: false,
            },
        )
        .expect("run preflight");

        assert_eq!(code, ExitCode::SUCCESS);
    }
}
