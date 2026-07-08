use crate::commands::{
    AbsolutePathPolicyArg, CacheModeArg, DagCli, MaterializeModeArg, ResumeFailureModeArg,
    RunProgressArg, RunTimeoutBehaviorArg,
};
use crate::graph_helpers::{
    parse_selectors, resolve_downstream_run_selection, resolve_upstream_run_selection,
    validate_partial_selection_surface,
};
use crate::output_contract::emit_json_line;
use crate::routes::plan_routes::{
    concise_plan_lines, plan_explain_payload, resolve_plan_preview_layout,
};
use crate::routes::policy_surface::{cache_surface_payload, policy_surface_payload};
use crate::routes::preconditions::{require_file, require_safe_path};
use crate::routes::resource_capacity_args::parse_resource_capacities;
use crate::routes::run_progress::{CompactRunProgressMonitor, JsonRunProgressMonitor};
use crate::run_data::map_materialize_mode;
use crate::runtime_inputs::{bind_runtime_inputs, missing_required_graph_inputs};
use crate::{
    emit_json, format_run_completion_human, load_graphs_or_emit, run_completion_summary, ExitCode,
};
use bijux_dag_runtime::{
    build_planner_analysis, registered_adapters, CacheMode, PlannerGuardrails, RunTimeoutBehavior,
    Runtime, RuntimeConfig,
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
    pub resume_run: Option<String>,
    pub resume_failure_mode: ResumeFailureModeArg,
    pub latest: Option<PathBuf>,
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub memory_budget_mb: Option<u32>,
    pub gpu_device_budget: Option<u32>,
    pub resource_capacity: &'a Vec<String>,
    pub node_timeout_ms: Option<u64>,
    pub run_timeout_ms: Option<u64>,
    pub run_timeout_behavior: RunTimeoutBehaviorArg,
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    pub hermetic: bool,
    pub select: &'a Vec<String>,
    pub exclude: &'a Vec<String>,
    pub to_node: &'a Vec<String>,
    pub dependency_closure: bool,
    pub materialize_inputs: MaterializeModeArg,
    pub cache: CacheModeArg,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicyArg,
    pub preflight_only: bool,
    pub explain_scheduling: bool,
    pub progress: RunProgressArg,
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

fn build_run_runtime_options(
    req: &RunRouteRequest<'_>,
    named_resource_capacities: std::collections::BTreeMap<String, u32>,
    preview_layout: Option<&bijux_dag_artifacts::RunDirLayout>,
    selectors: bijux_dag_runtime::SelectorSet,
    cache_dir: Option<PathBuf>,
    remote_cache_dir: Option<PathBuf>,
    absolute_path_policy: bijux_dag_runtime::AbsolutePathPolicy,
    policy: bijux_dag_runtime::PolicyConfig,
    upstream_selection_targets: Vec<String>,
    downstream_selection_roots: Vec<String>,
) -> RuntimeConfig {
    RuntimeConfig {
        jobs: req.jobs,
        cpu_budget: req.cpu_budget,
        memory_budget_mb: req.memory_budget_mb,
        gpu_device_budget: req.gpu_device_budget,
        named_resource_capacities,
        run_timeout_ms: req.run_timeout_ms,
        run_timeout_behavior: match req.run_timeout_behavior {
            RunTimeoutBehaviorArg::FinishRunning => RunTimeoutBehavior::FinishRunning,
            RunTimeoutBehaviorArg::CancelRunning => RunTimeoutBehavior::CancelRunning,
        },
        node_timeout_ms: req.node_timeout_ms,
        materialize_inputs: map_materialize_mode(req.materialize_inputs),
        cache_mode: match req.cache {
            CacheModeArg::Off => CacheMode::Off,
            CacheModeArg::Read => CacheMode::Read,
            CacheModeArg::Readwrite => CacheMode::ReadWrite,
        },
        cache_dir,
        remote_cache_dir,
        run_root: Some(req.out.to_path_buf()),
        absolute_path_policy,
        run_id: preview_layout.map(|layout| layout.run_id.clone()),
        resume_run_id: req.resume_run.clone(),
        resume_failure_mode: match req.resume_failure_mode {
            ResumeFailureModeArg::RerunIncomplete => {
                bijux_dag_runtime::ResumeFailureMode::RerunIncomplete
            }
            ResumeFailureModeArg::RejectIncomplete => {
                bijux_dag_runtime::ResumeFailureMode::RejectIncomplete
            }
        },
        latest_symlink: req.latest.clone(),
        policy,
        selectors,
        upstream_selection_targets,
        downstream_selection_roots,
        partial_rerun_dependency_closure: req.dependency_closure,
        scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
            max_parallelism: req.jobs.max(1),
            ..bijux_dag_runtime::SchedulerPolicy::default()
        },
        ..RuntimeConfig::default()
    }
}

fn load_resume_summary(run_dir: &Path) -> Option<bijux_dag_runtime::ResumeSummary> {
    let attempts_path = run_dir.join("run.attempts.json");
    let raw = fs::read_to_string(attempts_path).ok()?;
    let attempts = serde_json::from_str::<Vec<bijux_dag_runtime::RunAttempt>>(&raw).ok()?;
    attempts.last()?.resume_summary.clone()
}

enum RunProgressMonitor {
    Compact(CompactRunProgressMonitor),
    Json(JsonRunProgressMonitor),
}

impl RunProgressMonitor {
    fn finish(self) {
        match self {
            Self::Compact(monitor) => monitor.finish(),
            Self::Json(monitor) => monitor.finish(),
        }
    }

    fn streams_json(&self) -> bool {
        matches!(self, Self::Json(_))
    }
}

fn maybe_start_run_progress_monitor(
    cli: &DagCli,
    req: &RunRouteRequest<'_>,
    preview_layout: Option<&bijux_dag_artifacts::RunDirLayout>,
    fallback_total_nodes: usize,
) -> Option<RunProgressMonitor> {
    if cli.quiet || req.preflight_only || req.progress != RunProgressArg::Compact {
        return None;
    }
    let layout = preview_layout?;
    if cli.json {
        return Some(RunProgressMonitor::Json(JsonRunProgressMonitor::start(
            &layout.staging_path,
            &layout.final_path,
            fallback_total_nodes,
        )));
    }
    Some(RunProgressMonitor::Compact(CompactRunProgressMonitor::start(
        &layout.staging_path,
        &layout.final_path,
        fallback_total_nodes,
    )))
}

fn emit_run_execution_error(
    cli: &DagCli,
    streamed_json: bool,
    message: &str,
    completion_summary: Option<&serde_json::Value>,
    run_dir: Option<&Path>,
    payload: serde_json::Value,
) -> Result<ExitCode, ExitCode> {
    if cli.json {
        if streamed_json {
            emit_json_line("dag.run", false, payload, Vec::new(), ExitCode::from(3));
            return Err(ExitCode::from(3));
        }
        return emit_json(cli, "dag.run", false, payload, Vec::new(), ExitCode::from(3));
    }
    eprintln!("{message}");
    if let Some(summary) = completion_summary {
        eprintln!("{}", format_run_completion_human(summary));
    }
    if let Some(path) = run_dir {
        eprintln!("run dir: {}", path.display());
    }
    Err(ExitCode::from(3))
}

fn emit_run_json_result(
    cli: &DagCli,
    streamed_json: bool,
    payload: serde_json::Value,
) -> Result<ExitCode, ExitCode> {
    if streamed_json {
        emit_json_line("dag.run", true, payload, Vec::new(), ExitCode::SUCCESS);
        return Ok(ExitCode::SUCCESS);
    }
    emit_json(cli, "dag.run", true, payload, Vec::new(), ExitCode::SUCCESS)
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
                return emit_run_input_error(
                    cli,
                    &message,
                    json!({ "error": message, "error_class": "user" }),
                );
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
                "error_class": "user",
                "missing_inputs": missing_inputs,
            }),
        );
    }
    let runtime = Runtime::new();
    let named_resource_capacities = parse_resource_capacities(req.resource_capacity)?;
    let (deny_network, deny_clock, clean_env) =
        effective_policy_flags(req.deny_network, req.deny_clock, req.clean_env, req.hermetic);
    let deny_env = req.deny_env;
    validate_partial_selection_surface(
        &[],
        req.to_node,
        req.select,
        req.exclude,
        req.dependency_closure,
    )?;
    let (upstream_selection_targets, _) = resolve_upstream_run_selection(&graph, req.to_node)?;
    let (downstream_selection_roots, _) = resolve_downstream_run_selection(&graph, &[])?;
    let selectors =
        if upstream_selection_targets.is_empty() && downstream_selection_roots.is_empty() {
            parse_selectors(req.select, req.exclude)?
        } else {
            bijux_dag_runtime::SelectorSet::default()
        };
    let cache_dir = req.cache_dir.clone();
    let remote_cache_dir = req.remote_cache_dir.clone();
    let preview_layout = resolve_plan_preview_layout(
        Some(req.out),
        req.resume_run.as_deref().or(req.run_id.as_deref()),
    )?;
    let absolute_path_policy = req.absolute_path_policy.into();
    let options = build_run_runtime_options(
        &req,
        named_resource_capacities,
        preview_layout.as_ref(),
        selectors,
        cache_dir.clone(),
        remote_cache_dir,
        absolute_path_policy,
        bijux_dag_runtime::PolicyConfig {
            deny_network,
            deny_env,
            deny_clock,
            clean_env,
            ..bijux_dag_runtime::PolicyConfig::default()
        },
        upstream_selection_targets.clone(),
        downstream_selection_roots.clone(),
    );
    let cache_surface = cache_surface_payload(&options);
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
            "cache": {
                "local_preflight": cache_preflight(req.cache, &cache_dir),
                "surface": cache_surface.clone(),
            },
            "run_layout": preview_layout,
            "resume": req.resume_run.as_ref().map(|run_id| {
                json!({
                    "run_id": run_id,
                    "failure_mode": match req.resume_failure_mode {
                        ResumeFailureModeArg::RerunIncomplete => "rerun_incomplete",
                        ResumeFailureModeArg::RejectIncomplete => "reject_incomplete",
                    },
                })
            }),
            "policy": {
                "deny_network": options.policy.deny_network,
                "deny_env": options.policy.deny_env,
                "deny_clock": options.policy.deny_clock,
                "clean_env": options.policy.clean_env,
                "container_image_reference_policy": match options
                    .policy
                    .container_image_reference_policy
                {
                    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest => {
                        "require_digest"
                    }
                    bijux_dag_runtime::ContainerImageReferencePolicy::AllowUnpinned => {
                        "allow_unpinned"
                    }
                },
            },
            "policy_surface": policy_surface_payload(&graph, &options, req.hermetic)?,
            "input_summary": runtime_inputs.human_summary,
            "redacted_input_keys": runtime_inputs.redacted_keys,
            "selectors": {
                "include": req.select,
                "exclude": req.exclude,
                "upstream_targets": upstream_selection_targets,
                "downstream_roots": downstream_selection_roots,
                "dependency_closure": req.dependency_closure,
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
    let progress_monitor =
        maybe_start_run_progress_monitor(cli, &req, preview_layout.as_ref(), graph.nodes.len());
    let streamed_json_progress =
        progress_monitor.as_ref().is_some_and(RunProgressMonitor::streams_json);
    let run_result = runtime.run(&graph, req.out, options);
    if let Some(progress_monitor) = progress_monitor {
        progress_monitor.finish();
    }
    let run_path = match run_result {
        Ok(path) => path,
        Err(error) => {
            let resume_summary = preview_layout
                .as_ref()
                .and_then(|layout| load_resume_summary(&layout.staging_path));
            let completion_summary = preview_layout.as_ref().and_then(|layout| {
                if layout.staging_path.exists() {
                    run_completion_summary(&layout.staging_path).ok()
                } else {
                    None
                }
            });
            return emit_run_execution_error(
                cli,
                streamed_json_progress,
                &error.to_string(),
                completion_summary.as_ref(),
                preview_layout.as_ref().map(|layout| layout.staging_path.as_path()),
                json!({
                    "error": error.to_string(),
                    "error_class": "runtime",
                    "run_layout": preview_layout,
                    "resume_summary": resume_summary,
                    "summary": completion_summary,
                }),
            );
        }
    };
    let resume_summary = load_resume_summary(&run_path);
    let completion_summary = run_completion_summary(&run_path).map_err(|_| ExitCode::from(3))?;

    if cli.json {
        return emit_run_json_result(
            cli,
            streamed_json_progress,
            json!({
                "run_dir": run_path,
                "cache": cache_surface,
                "run_layout": preview_layout,
                "resume_summary": resume_summary,
                "summary": completion_summary,
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
        if let Some(summary) = resume_summary.as_ref() {
            println!(
                "resume: reused={} rerun={} rejected={}",
                summary.reused_nodes.len(),
                summary.rerun_nodes.len(),
                summary.rejected_nodes.len(),
            );
        }
        println!("{}", format_run_completion_human(&completion_summary));
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
        build_run_runtime_options, cache_preflight, effective_policy_flags, emit_run_input_error,
        handle_run_command, load_resume_summary, maybe_start_run_progress_monitor,
        RunProgressMonitor, RunRouteRequest,
    };
    use crate::commands::{
        AbsolutePathPolicyArg, CacheModeArg, Commands, DagCli, MaterializeModeArg,
        ResumeFailureModeArg, RunProgressArg, RunTimeoutBehaviorArg,
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
    fn progress_monitor_respects_quiet_json_and_preflight_modes() {
        let dir = tempfile::tempdir().expect("tmp");
        let layout = super::resolve_plan_preview_layout(Some(dir.path()), Some("progress-run"))
            .expect("layout");
        let request = RunRouteRequest {
            dags: &[],
            out: dir.path(),
            input: &Vec::new(),
            inputs_file: None,
            run_id: Some("progress-run".to_string()),
            resume_run: None,
            resume_failure_mode: ResumeFailureModeArg::RerunIncomplete,
            latest: None,
            jobs: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            resource_capacity: &Vec::new(),
            node_timeout_ms: None,
            run_timeout_ms: None,
            run_timeout_behavior: RunTimeoutBehaviorArg::FinishRunning,
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: false,
            hermetic: false,
            select: &Vec::new(),
            exclude: &Vec::new(),
            to_node: &Vec::new(),
            dependency_closure: false,
            materialize_inputs: MaterializeModeArg::Copy,
            cache: CacheModeArg::Off,
            cache_dir: None,
            remote_cache_dir: None,
            absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
            preflight_only: false,
            explain_scheduling: false,
            progress: RunProgressArg::Compact,
        };

        let quiet_cli = DagCli { json: false, quiet: true, command: Commands::Version };
        let json_cli = DagCli { json: true, quiet: false, command: Commands::Version };

        assert!(
            maybe_start_run_progress_monitor(&quiet_cli, &request, layout.as_ref(), 3).is_none()
        );
        assert!(matches!(
            maybe_start_run_progress_monitor(&json_cli, &request, layout.as_ref(), 3),
            Some(RunProgressMonitor::Json(_))
        ));
        assert!(maybe_start_run_progress_monitor(
            &DagCli { json: false, quiet: false, command: Commands::Version },
            &RunRouteRequest { preflight_only: true, ..request },
            layout.as_ref(),
            3,
        )
        .is_none());
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
                resume_run: None,
                resume_failure_mode: ResumeFailureModeArg::RerunIncomplete,
                latest: None,
                jobs: 1,
                cpu_budget: None,
                memory_budget_mb: None,
                gpu_device_budget: None,
                resource_capacity: &Vec::new(),
                node_timeout_ms: None,
                run_timeout_ms: None,
                run_timeout_behavior: RunTimeoutBehaviorArg::FinishRunning,
                deny_network: false,
                deny_env: false,
                deny_clock: false,
                clean_env: false,
                hermetic: false,
                select: &Vec::new(),
                exclude: &Vec::new(),
                to_node: &Vec::new(),
                dependency_closure: false,
                materialize_inputs: MaterializeModeArg::Copy,
                cache: CacheModeArg::Off,
                cache_dir: None,
                remote_cache_dir: None,
                absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
                preflight_only: true,
                explain_scheduling: false,
                progress: RunProgressArg::Off,
            },
        )
        .expect("run preflight");

        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn run_preflight_accepts_upstream_target_mode() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        let out = dir.path().join("runs");
        let to_node = vec!["publish".to_string()];
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"extract","kind":"const","outputs":[{"name":"report","path":"extract/report.json"}],"params":{"value":"seed"}},
                {"id":"publish","kind":"const","inputs":["report"],"outputs":[{"name":"out","path":"publish/out.json"}],"params":{"seed":{"node_output":{"node_id":"extract","output_name":"report"}}}}
              ],
              "edges":[{"from":{"node_id":"extract","port":"report"},"to":{"node_id":"publish","port":"report"}}]
            }"#,
        )
        .expect("write graph");

        let cli = DagCli { json: true, quiet: true, command: Commands::Version };
        let code = handle_run_command(
            &cli,
            RunRouteRequest {
                dags: &[dag],
                out: &out,
                input: &Vec::new(),
                inputs_file: None,
                run_id: Some("previewed".to_string()),
                resume_run: None,
                resume_failure_mode: ResumeFailureModeArg::RerunIncomplete,
                latest: None,
                jobs: 1,
                cpu_budget: None,
                memory_budget_mb: None,
                gpu_device_budget: None,
                resource_capacity: &Vec::new(),
                node_timeout_ms: None,
                run_timeout_ms: None,
                run_timeout_behavior: RunTimeoutBehaviorArg::FinishRunning,
                deny_network: false,
                deny_env: false,
                deny_clock: false,
                clean_env: false,
                hermetic: false,
                select: &Vec::new(),
                exclude: &Vec::new(),
                to_node: &to_node,
                dependency_closure: false,
                materialize_inputs: MaterializeModeArg::Copy,
                cache: CacheModeArg::Off,
                cache_dir: None,
                remote_cache_dir: None,
                absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
                preflight_only: true,
                explain_scheduling: false,
                progress: RunProgressArg::Off,
            },
        )
        .expect("run preflight");

        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn run_runtime_options_preserve_selector_and_closure_configuration() {
        let out_dir = tempfile::tempdir().expect("tmp");
        let request = RunRouteRequest {
            dags: &[],
            out: out_dir.path(),
            input: &Vec::new(),
            inputs_file: None,
            run_id: Some("selected-run".to_string()),
            resume_run: Some("selected-run".to_string()),
            resume_failure_mode: ResumeFailureModeArg::RejectIncomplete,
            latest: None,
            jobs: 3,
            cpu_budget: Some(4),
            memory_budget_mb: Some(4096),
            gpu_device_budget: Some(2),
            resource_capacity: &vec!["database_slot=2".to_string(), "license.render=1".to_string()],
            node_timeout_ms: Some(10),
            run_timeout_ms: Some(20),
            run_timeout_behavior: RunTimeoutBehaviorArg::CancelRunning,
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
            hermetic: false,
            select: &Vec::new(),
            exclude: &Vec::new(),
            to_node: &Vec::new(),
            dependency_closure: true,
            materialize_inputs: MaterializeModeArg::Hardlink,
            cache: CacheModeArg::Readwrite,
            cache_dir: Some(out_dir.path().join("cache")),
            remote_cache_dir: Some(out_dir.path().join("remote-cache")),
            absolute_path_policy: AbsolutePathPolicyArg::AllowLiteral,
            preflight_only: false,
            explain_scheduling: false,
            progress: RunProgressArg::Off,
        };
        let selectors = bijux_dag_runtime::SelectorSet {
            include: vec![bijux_dag_runtime::Selector::Id("train".to_string())],
            exclude: vec![bijux_dag_runtime::Selector::Kind("const".to_string())],
        };
        let layout = super::resolve_plan_preview_layout(Some(out_dir.path()), Some("selected-run"))
            .expect("layout");
        let options = build_run_runtime_options(
            &request,
            super::parse_resource_capacities(request.resource_capacity)
                .expect("resource capacities"),
            layout.as_ref(),
            selectors.clone(),
            request.cache_dir.clone(),
            request.remote_cache_dir.clone(),
            bijux_dag_runtime::AbsolutePathPolicy::AllowLiteral,
            bijux_dag_runtime::PolicyConfig {
                deny_network: true,
                deny_env: false,
                deny_clock: true,
                clean_env: true,
                ..bijux_dag_runtime::PolicyConfig::default()
            },
            vec!["report".to_string()],
            Vec::new(),
        );

        assert_eq!(options.jobs, 3);
        assert_eq!(options.scheduler_policy.max_parallelism, 3);
        assert_eq!(options.cpu_budget, Some(4));
        assert_eq!(options.memory_budget_mb, Some(4096));
        assert_eq!(options.gpu_device_budget, Some(2));
        assert_eq!(options.named_resource_capacities.get("database_slot"), Some(&2));
        assert_eq!(options.named_resource_capacities.get("license.render"), Some(&1));
        assert_eq!(
            options.run_timeout_behavior,
            bijux_dag_runtime::RunTimeoutBehavior::CancelRunning
        );
        assert!(options.partial_rerun_dependency_closure);
        assert_eq!(options.run_id.as_deref(), Some("selected-run"));
        assert_eq!(options.resume_run_id.as_deref(), Some("selected-run"));
        assert_eq!(
            options.resume_failure_mode,
            bijux_dag_runtime::ResumeFailureMode::RejectIncomplete
        );
        assert_eq!(options.selectors.include.len(), selectors.include.len());
        assert_eq!(options.selectors.exclude.len(), selectors.exclude.len());
        assert_eq!(options.upstream_selection_targets, vec!["report".to_string()]);
        assert!(matches!(options.materialize_inputs, bijux_dag_runtime::MaterializeMode::Hardlink));
        assert!(matches!(options.cache_mode, bijux_dag_runtime::CacheMode::ReadWrite));
        assert!(options.policy.deny_network);
        assert!(options.policy.deny_clock);
    }

    #[test]
    fn load_resume_summary_reads_latest_attempt_summary() {
        let dir = tempfile::tempdir().expect("tmp");
        let attempts = vec![
            bijux_dag_runtime::RunAttempt {
                attempt_index: 1,
                run_id: bijux_dag_runtime::RunId("resume-run".to_string()),
                parent_run_id: None,
                reason: "initial_submission".to_string(),
                resume_summary: None,
            },
            bijux_dag_runtime::RunAttempt {
                attempt_index: 2,
                run_id: bijux_dag_runtime::RunId("resume-run".to_string()),
                parent_run_id: None,
                reason: "resume".to_string(),
                resume_summary: Some(bijux_dag_runtime::ResumeSummary {
                    failure_mode: bijux_dag_runtime::ResumeFailureMode::RerunIncomplete,
                    reused_nodes: vec!["extract".to_string()],
                    rerun_nodes: vec!["publish".to_string()],
                    rejected_nodes: Vec::new(),
                }),
            },
        ];
        fs::write(
            dir.path().join("run.attempts.json"),
            serde_json::to_vec_pretty(&attempts).expect("serialize attempts"),
        )
        .expect("write attempts");

        let summary = load_resume_summary(dir.path()).expect("resume summary");
        assert_eq!(summary.reused_nodes, vec!["extract"]);
        assert_eq!(summary.rerun_nodes, vec!["publish"]);
        assert!(summary.rejected_nodes.is_empty());
    }
}
