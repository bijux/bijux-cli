use crate::cache::verify_cache_entry_cli;
use crate::integrity_service::verify_run;
use crate::read_run_id;
use crate::replay_service::{
    node_rerun_diff_report, verify_replay_boundary_inputs, ReplayBoundaryVerificationReport,
};
use crate::run_data::env_cache_dir;
use crate::run_data::load_snapshot;
use crate::{read_file, ExitCode, Runtime};
use bijux_dag_artifacts::{
    sha256_artifact_path, write_run_outputs_index, AdapterInfo, Manifest, NodeCounts, NodeTrace,
    OutputSummary, OutputsIndex, PolicyInfo, RunMetadata, RunOutputFile, RunOutputsIndex,
    RunSummary,
};
use bijux_dag_core::Graph;
use bijux_dag_runtime::{
    CacheMode, MaterializeMode, PolicyConfig, RunSnapshot, RunState, RuntimeConfig, SchedulerPolicy,
};
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct RepairExecutionOptions {
    pub(crate) out_dir: Option<PathBuf>,
    pub(crate) run_id: Option<String>,
    pub(crate) jobs: usize,
    pub(crate) materialize_inputs: MaterializeMode,
    pub(crate) cache_mode: CacheMode,
    pub(crate) cache_dir: Option<PathBuf>,
    pub(crate) remote_cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunRepairIssueKind {
    FailedNode,
    MissingOutput,
    CorruptArtifact,
    MissingTrace,
    MissingOutputsIndex,
    MissingRunOutputsIndexEntry,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunRepairIssue {
    pub(crate) kind: RunRepairIssueKind,
    pub(crate) node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) output_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) status: Option<String>,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RunRepairActionKind {
    RebuildManifest,
    RebuildRunLogIndex,
    RestoreFromCache,
    RerunDownstreamClosure,
    VerifyRepairRun,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunRepairAction {
    pub(crate) kind: RunRepairActionKind,
    pub(crate) summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) node_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) affected_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunRepairMetadataReport {
    pub(crate) manifest_path: String,
    pub(crate) index_path: String,
    pub(crate) manifest_valid: bool,
    pub(crate) index_valid: bool,
    pub(crate) manifest_rewritten: bool,
    pub(crate) index_rewritten: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunRepairCacheRecovery {
    pub(crate) node_id: String,
    pub(crate) cache_key: String,
    pub(crate) cache_entry_path: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) restored_outputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) restored_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunRepairExecutionReport {
    pub(crate) out_dir: String,
    pub(crate) run_dir: String,
    pub(crate) run_id: String,
    pub(crate) boundary_verification: ReplayBoundaryVerificationReport,
    pub(crate) verify_report: Value,
    pub(crate) verified: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) node_rerun_diffs: Vec<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunRepairReport {
    pub(crate) run_id: String,
    pub(crate) source_run_dir: String,
    pub(crate) source_status: String,
    pub(crate) incomplete_marker_present: bool,
    pub(crate) metadata: RunRepairMetadataReport,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) issues: Vec<RunRepairIssue>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) cache_recovery_candidates: Vec<RunRepairCacheRecovery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) cache_recoveries_applied: Vec<RunRepairCacheRecovery>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) repair_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) invalidated_nodes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) proposed_actions: Vec<RunRepairAction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) blocking_issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) boundary_verification: Option<ReplayBoundaryVerificationReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) repair_run: Option<RunRepairExecutionReport>,
}

#[derive(Debug)]
struct RepairAnalysis {
    run_id: String,
    source_status: String,
    metadata: RunRepairMetadataReport,
    incomplete_marker_present: bool,
    graph: Option<Graph>,
    manifest: Option<Manifest>,
    run_snapshot: Option<RunSnapshot>,
    issues: Vec<RunRepairIssue>,
    cache_recovery_candidates: Vec<RunRepairCacheRecovery>,
    repair_roots: Vec<String>,
    invalidated_nodes: Vec<String>,
    proposed_actions: Vec<RunRepairAction>,
    blocking_issues: Vec<String>,
    boundary_verification: Option<ReplayBoundaryVerificationReport>,
}

#[derive(Debug)]
struct RepairNodeContext {
    trace: Option<NodeTrace>,
    outputs_index: Option<bijux_dag_artifacts::OutputsIndex>,
}

pub(crate) fn plan_run_repair(
    run_dir: &Path,
    cache_dir: Option<PathBuf>,
    remote_cache_dir: Option<PathBuf>,
) -> Result<RunRepairReport, ExitCode> {
    let analysis = analyze_run_repair(run_dir, cache_dir, remote_cache_dir)?;
    Ok(analysis_into_report(run_dir, analysis, None, Vec::new()))
}

pub(crate) fn apply_run_repair(
    run_dir: &Path,
    options: &RepairExecutionOptions,
) -> Result<RunRepairReport, ExitCode> {
    let mut analysis =
        analyze_run_repair(run_dir, options.cache_dir.clone(), options.remote_cache_dir.clone())?;
    let metadata = repair_metadata(run_dir)?;
    analysis.metadata = metadata;
    let mut cache_recoveries_applied = Vec::new();

    if !analysis.cache_recovery_candidates.is_empty() {
        cache_recoveries_applied =
            apply_cache_recoveries(run_dir, &analysis.cache_recovery_candidates)?;
        if !cache_recoveries_applied.is_empty() {
            rebuild_run_outputs_index_from_run_dir(run_dir)?;
            let metadata = repair_metadata(run_dir)?;
            analysis = analyze_run_repair(
                run_dir,
                options.cache_dir.clone(),
                options.remote_cache_dir.clone(),
            )?;
            analysis.metadata = metadata;
        }
    }

    if analysis.repair_roots.is_empty() {
        return Ok(analysis_into_report(run_dir, analysis, None, cache_recoveries_applied));
    }
    let out_dir = resolve_repair_out_dir(run_dir, options)?;
    if !analysis.blocking_issues.is_empty() {
        return Ok(analysis_into_report(run_dir, analysis, None, cache_recoveries_applied));
    }
    let boundary = analysis.boundary_verification.clone().ok_or_else(|| ExitCode::from(3))?;
    if !boundary.verified {
        return Ok(analysis_into_report(run_dir, analysis, None, cache_recoveries_applied));
    }
    let graph = analysis.graph.as_ref().ok_or_else(|| ExitCode::from(3))?;
    let runtime = Runtime::new();
    let run_path = runtime
        .run(
            graph,
            &out_dir,
            build_repair_runtime_options(&analysis, analysis.manifest.as_ref(), options, run_dir),
        )
        .map_err(|_| ExitCode::from(3))?;
    let verify_report = verify_run(&run_path, true, true)?;
    let verified = verify_report.get("status").and_then(Value::as_str) == Some("ok")
        && verify_report
            .get("errors")
            .and_then(Value::as_array)
            .map(|errors| errors.is_empty())
            .unwrap_or(false);
    let mut node_rerun_diffs = Vec::new();
    for node_id in &analysis.repair_roots {
        node_rerun_diffs.push(node_rerun_diff_report(run_dir, &run_path, node_id)?);
    }
    let execution = RunRepairExecutionReport {
        out_dir: out_dir.display().to_string(),
        run_dir: run_path.display().to_string(),
        run_id: read_run_id(&run_path)?,
        boundary_verification: boundary,
        verify_report,
        verified,
        node_rerun_diffs,
    };
    Ok(analysis_into_report(run_dir, analysis, Some(execution), cache_recoveries_applied))
}

pub(crate) fn run_repair_ok(report: &RunRepairReport, apply: bool) -> bool {
    let metadata_ok = report.metadata.manifest_valid && report.metadata.index_valid;
    if apply {
        if report.issues.is_empty() {
            return metadata_ok;
        }
        return metadata_ok
            && report.blocking_issues.is_empty()
            && report
                .repair_run
                .as_ref()
                .map(|execution| execution.verified)
                .unwrap_or(report.issues.is_empty());
    }
    metadata_ok && report.issues.is_empty() && report.blocking_issues.is_empty()
}

fn analysis_into_report(
    run_dir: &Path,
    analysis: RepairAnalysis,
    repair_run: Option<RunRepairExecutionReport>,
    cache_recoveries_applied: Vec<RunRepairCacheRecovery>,
) -> RunRepairReport {
    RunRepairReport {
        run_id: analysis.run_id,
        source_run_dir: run_dir.display().to_string(),
        source_status: analysis.source_status,
        incomplete_marker_present: analysis.incomplete_marker_present,
        metadata: analysis.metadata,
        issues: analysis.issues,
        cache_recovery_candidates: analysis.cache_recovery_candidates,
        cache_recoveries_applied,
        repair_roots: analysis.repair_roots,
        invalidated_nodes: analysis.invalidated_nodes,
        proposed_actions: analysis.proposed_actions,
        blocking_issues: analysis.blocking_issues,
        boundary_verification: analysis.boundary_verification,
        repair_run,
    }
}

fn analyze_run_repair(
    run_dir: &Path,
    cache_dir: Option<PathBuf>,
    remote_cache_dir: Option<PathBuf>,
) -> Result<RepairAnalysis, ExitCode> {
    let metadata = inspect_metadata(run_dir)?;
    let incomplete_marker_present = run_dir.join(".run-incomplete.json").exists();
    let run_snapshot = read_run_snapshot(run_dir)?;
    let run_id = read_run_id_with_fallback(run_dir, run_snapshot.as_ref())?;
    let source_status = read_manifest_status(run_dir)?;
    let manifest = read_manifest_tolerant(run_dir)?;

    let mut blocking_issues = Vec::new();
    let graph_snapshot = match load_snapshot(run_dir) {
        Ok(snapshot) => Some(snapshot),
        Err(_) => {
            blocking_issues.push(
                "graph.snapshot.json is missing or unreadable, so repair rerun planning is unavailable"
                    .to_string(),
            );
            None
        }
    };
    let graph = graph_snapshot.as_ref().map(|snapshot| snapshot.graph.clone());
    let graph_ref = graph.as_ref();

    let node_contexts = collect_node_contexts(run_dir)?;
    let run_outputs_index = read_run_outputs_index_tolerant(run_dir)?;
    let mut issues = Vec::new();

    if let Some(graph) = graph_ref {
        for node in &graph.nodes {
            let context = node_contexts.get(&node.id);
            if context.and_then(|entry| entry.trace.as_ref()).is_none() {
                issues.push(RunRepairIssue {
                    kind: RunRepairIssueKind::MissingTrace,
                    node_id: node.id.clone(),
                    output_name: None,
                    path: None,
                    status: None,
                    detail: "node trace is missing, so the node outcome cannot be trusted"
                        .to_string(),
                });
                continue;
            }
            let trace = context.and_then(|entry| entry.trace.as_ref()).expect("trace checked");
            let status = trace.status.clone();
            if is_repair_failure_status(&status) {
                issues.push(RunRepairIssue {
                    kind: RunRepairIssueKind::FailedNode,
                    node_id: node.id.clone(),
                    output_name: None,
                    path: None,
                    status: Some(status),
                    detail: "node did not finish in a reusable success or cache state".to_string(),
                });
                continue;
            }
            if !matches!(status.as_str(), "success" | "cached") {
                continue;
            }
            let Some(outputs_index) = context.and_then(|entry| entry.outputs_index.as_ref()) else {
                issues.push(RunRepairIssue {
                    kind: RunRepairIssueKind::MissingOutputsIndex,
                    node_id: node.id.clone(),
                    output_name: None,
                    path: Some(
                        run_dir
                            .join("nodes")
                            .join(&node.id)
                            .join("outputs")
                            .join("index.json")
                            .display()
                            .to_string(),
                    ),
                    status: Some(status),
                    detail: "successful node is missing outputs/index.json".to_string(),
                });
                continue;
            };

            for output in node.outputs.iter().filter(|output| output.required) {
                let indexed_output =
                    outputs_index.files.iter().find(|file| file.name == output.name);
                let Some(indexed_output) = indexed_output else {
                    issues.push(RunRepairIssue {
                        kind: RunRepairIssueKind::MissingOutput,
                        node_id: node.id.clone(),
                        output_name: Some(output.name.clone()),
                        path: Some(output.path.clone()),
                        status: Some(status.clone()),
                        detail: "required output is missing from node outputs index".to_string(),
                    });
                    continue;
                };
                let output_path =
                    run_dir.join("nodes").join(&node.id).join("outputs").join(&indexed_output.path);
                if fs::metadata(&output_path).is_err() {
                    issues.push(RunRepairIssue {
                        kind: RunRepairIssueKind::MissingOutput,
                        node_id: node.id.clone(),
                        output_name: Some(indexed_output.name.clone()),
                        path: Some(output_path.display().to_string()),
                        status: Some(status.clone()),
                        detail: "required output payload is missing from the run directory"
                            .to_string(),
                    });
                    continue;
                }
                let actual_sha256 =
                    sha256_artifact_path(&output_path).map_err(|_| ExitCode::from(3))?;
                if actual_sha256 != indexed_output.sha256 {
                    issues.push(RunRepairIssue {
                        kind: RunRepairIssueKind::CorruptArtifact,
                        node_id: node.id.clone(),
                        output_name: Some(indexed_output.name.clone()),
                        path: Some(output_path.display().to_string()),
                        status: Some(status.clone()),
                        detail: format!(
                            "required output hash mismatch: expected {}, found {}",
                            indexed_output.sha256, actual_sha256
                        ),
                    });
                }
                if !run_outputs_contains(
                    &run_outputs_index,
                    &node.id,
                    &indexed_output.name,
                    &indexed_output.path,
                ) {
                    issues.push(RunRepairIssue {
                        kind: RunRepairIssueKind::MissingRunOutputsIndexEntry,
                        node_id: node.id.clone(),
                        output_name: Some(indexed_output.name.clone()),
                        path: Some(indexed_output.path.clone()),
                        status: Some(status.clone()),
                        detail: "required output is absent from the run outputs index".to_string(),
                    });
                }
            }
        }
    }

    if let Some(run_outputs_index) = run_outputs_index.as_ref() {
        for file in &run_outputs_index.files {
            let path = run_dir.join(&file.path);
            if fs::metadata(&path).is_err() {
                issues.push(RunRepairIssue {
                    kind: RunRepairIssueKind::MissingOutput,
                    node_id: file.node_id.clone(),
                    output_name: Some(file.name.clone()),
                    path: Some(path.display().to_string()),
                    status: node_contexts
                        .get(&file.node_id)
                        .and_then(|entry| entry.trace.as_ref())
                        .map(|trace| trace.status.clone()),
                    detail: "run outputs index points at a missing artifact payload".to_string(),
                });
                continue;
            }
            let actual_sha256 = sha256_artifact_path(&path).map_err(|_| ExitCode::from(3))?;
            if actual_sha256 != file.sha256 {
                issues.push(RunRepairIssue {
                    kind: RunRepairIssueKind::CorruptArtifact,
                    node_id: file.node_id.clone(),
                    output_name: Some(file.name.clone()),
                    path: Some(path.display().to_string()),
                    status: node_contexts
                        .get(&file.node_id)
                        .and_then(|entry| entry.trace.as_ref())
                        .map(|trace| trace.status.clone()),
                    detail: format!(
                        "run outputs index hash mismatch: expected {}, found {}",
                        file.sha256, actual_sha256
                    ),
                });
            }
        }
    }

    issues.sort_by(|left, right| {
        (
            left.node_id.clone(),
            left.output_name.clone().unwrap_or_default(),
            left.path.clone().unwrap_or_default(),
            serde_json::to_string(&left.kind).unwrap_or_default(),
        )
            .cmp(&(
                right.node_id.clone(),
                right.output_name.clone().unwrap_or_default(),
                right.path.clone().unwrap_or_default(),
                serde_json::to_string(&right.kind).unwrap_or_default(),
            ))
    });
    issues.dedup();

    let cache_recovery_candidates = collect_cache_recovery_candidates(
        manifest.as_ref(),
        &node_contexts,
        &issues,
        cache_dir,
        remote_cache_dir,
    );

    let repair_roots = if let Some(graph) = graph_ref {
        minimal_repair_roots(graph, &issues, &cache_recovery_candidates)
    } else {
        Vec::new()
    };
    let invalidated_nodes = if let Some(graph) = graph_ref {
        let mut nodes = bijux_dag_runtime::compute_downstream_run_closure(graph, &repair_roots)
            .into_iter()
            .collect::<Vec<_>>();
        nodes.sort();
        nodes
    } else {
        Vec::new()
    };
    let boundary_verification = if !repair_roots.is_empty() {
        Some(verify_replay_boundary_inputs(run_dir, &run_id, &repair_roots)?)
    } else {
        None
    };
    let proposed_actions = build_proposed_actions(
        &metadata,
        &cache_recovery_candidates,
        &repair_roots,
        &invalidated_nodes,
        !issues.is_empty(),
    );

    Ok(RepairAnalysis {
        run_id,
        source_status,
        metadata,
        incomplete_marker_present,
        graph,
        manifest,
        run_snapshot,
        issues,
        cache_recovery_candidates,
        repair_roots,
        invalidated_nodes,
        proposed_actions,
        blocking_issues,
        boundary_verification,
    })
}

fn build_proposed_actions(
    metadata: &RunRepairMetadataReport,
    cache_recovery_candidates: &[RunRepairCacheRecovery],
    repair_roots: &[String],
    invalidated_nodes: &[String],
    has_issues: bool,
) -> Vec<RunRepairAction> {
    let mut actions = Vec::new();
    if !metadata.manifest_valid {
        actions.push(RunRepairAction {
            kind: RunRepairActionKind::RebuildManifest,
            summary: "rebuild manifest.json from the run snapshot, traces, and outputs".to_string(),
            node_roots: Vec::new(),
            affected_nodes: Vec::new(),
        });
    }
    if !metadata.index_valid {
        actions.push(RunRepairAction {
            kind: RunRepairActionKind::RebuildRunLogIndex,
            summary: "rebuild run-log.index.json from run.log.jsonl".to_string(),
            node_roots: Vec::new(),
            affected_nodes: Vec::new(),
        });
    }
    let cache_restorable_nodes = cache_recovery_candidates
        .iter()
        .map(|candidate| candidate.node_id.clone())
        .collect::<Vec<_>>();
    if !cache_restorable_nodes.is_empty() {
        actions.push(RunRepairAction {
            kind: RunRepairActionKind::RestoreFromCache,
            summary:
                "restore exact cached outputs for nodes whose retained cache proof still verifies"
                    .to_string(),
            node_roots: cache_restorable_nodes.clone(),
            affected_nodes: cache_restorable_nodes,
        });
    }
    if has_issues && !repair_roots.is_empty() {
        actions.push(RunRepairAction {
            kind: RunRepairActionKind::RerunDownstreamClosure,
            summary: "spawn a child repair run that reruns the damaged nodes and their downstream closure"
                .to_string(),
            node_roots: repair_roots.to_vec(),
            affected_nodes: invalidated_nodes.to_vec(),
        });
        actions.push(RunRepairAction {
            kind: RunRepairActionKind::VerifyRepairRun,
            summary: "verify the child repair run in strict mode before trusting repaired outputs"
                .to_string(),
            node_roots: repair_roots.to_vec(),
            affected_nodes: invalidated_nodes.to_vec(),
        });
    }
    actions
}

fn minimal_repair_roots(
    graph: &Graph,
    issues: &[RunRepairIssue],
    cache_recovery_candidates: &[RunRepairCacheRecovery],
) -> Vec<String> {
    let cache_recovered_nodes = cache_recovery_candidates
        .iter()
        .map(|entry| entry.node_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut candidates = issues
        .iter()
        .filter(|issue| {
            matches!(
                issue.kind,
                RunRepairIssueKind::FailedNode
                    | RunRepairIssueKind::MissingOutput
                    | RunRepairIssueKind::CorruptArtifact
                    | RunRepairIssueKind::MissingOutputsIndex
                    | RunRepairIssueKind::MissingRunOutputsIndexEntry
            )
        })
        .filter(|issue| !cache_recovered_nodes.contains(issue.node_id.as_str()))
        .map(|issue| issue.node_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    candidates.sort();
    let mut minimal = Vec::new();
    for candidate in &candidates {
        let covered = candidates.iter().any(|other| {
            other != candidate
                && bijux_dag_runtime::compute_downstream_run_closure(
                    graph,
                    std::slice::from_ref(other),
                )
                .contains(candidate)
        });
        if !covered {
            minimal.push(candidate.clone());
        }
    }
    minimal
}

fn repair_metadata(run_dir: &Path) -> Result<RunRepairMetadataReport, ExitCode> {
    let mut report = inspect_metadata(run_dir)?;
    if !report.manifest_valid {
        let manifest = build_manifest_from_run_dir(run_dir)?;
        fs::write(
            Path::new(&report.manifest_path),
            serde_json::to_vec_pretty(&manifest).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        report.manifest_valid = true;
        report.manifest_rewritten = true;
        report.notes.push("manifest rebuilt from run snapshot, traces, and outputs".to_string());
    }
    if !report.index_valid {
        let rebuilt = rebuild_run_log_index(run_dir)?;
        fs::write(
            Path::new(&report.index_path),
            serde_json::to_vec_pretty(&rebuilt).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        report.index_valid = true;
        report.index_rewritten = true;
        report.notes.push("run log index rebuilt from event journal".to_string());
    }
    Ok(report)
}

fn inspect_metadata(run_dir: &Path) -> Result<RunRepairMetadataReport, ExitCode> {
    let manifest_path = run_dir.join("manifest.json");
    let index_path = run_dir.join("run-log.index.json");
    Ok(RunRepairMetadataReport {
        manifest_path: manifest_path.display().to_string(),
        index_path: index_path.display().to_string(),
        manifest_valid: manifest_path.exists() && read_json_value(&manifest_path).is_ok(),
        index_valid: index_path.exists()
            && fs::read(&index_path)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Vec<Value>>(&bytes).ok())
                .is_some(),
        manifest_rewritten: false,
        index_rewritten: false,
        notes: Vec::new(),
    })
}

fn build_repair_runtime_options(
    analysis: &RepairAnalysis,
    manifest: Option<&Manifest>,
    options: &RepairExecutionOptions,
    source_run_dir: &Path,
) -> RuntimeConfig {
    let out_dir =
        options.out_dir.clone().or_else(|| source_run_dir.parent().map(Path::to_path_buf));
    let policy = manifest.map_or_else(PolicyConfig::default, |manifest| PolicyConfig {
        deny_network: manifest.policy.deny_network,
        deny_env: manifest.policy.deny_env,
        deny_clock: manifest.policy.deny_clock,
        clean_env: manifest.policy.clean_env,
        ..PolicyConfig::default()
    });
    RuntimeConfig {
        jobs: options.jobs.max(1),
        materialize_inputs: options.materialize_inputs,
        cache_mode: options.cache_mode.clone(),
        cache_dir: options.cache_dir.clone(),
        remote_cache_dir: options.remote_cache_dir.clone(),
        run_root: out_dir,
        run_id: options.run_id.clone(),
        parent_run_id: Some(analysis.run_id.clone()),
        replay_source_run_dir: Some(source_run_dir.to_path_buf()),
        submission_source: "repair".to_string(),
        trigger_source: "runtime_repair".to_string(),
        operator: analysis
            .run_snapshot
            .as_ref()
            .map(|snapshot| snapshot.operator.clone())
            .or_else(|| {
                manifest
                    .and_then(|entry| entry.run_metadata.as_ref())
                    .map(|entry| entry.operator.clone())
            })
            .unwrap_or_else(|| "repair".to_string()),
        labels: manifest
            .and_then(|entry| entry.run_metadata.as_ref())
            .map(|entry| entry.labels.clone())
            .unwrap_or_default(),
        policy,
        downstream_selection_roots: analysis.repair_roots.clone(),
        partial_rerun_dependency_closure: true,
        scheduler_policy: SchedulerPolicy {
            max_parallelism: options.jobs.max(1),
            ..SchedulerPolicy::default()
        },
        ..RuntimeConfig::default()
    }
}

fn resolve_repair_out_dir(
    run_dir: &Path,
    options: &RepairExecutionOptions,
) -> Result<PathBuf, ExitCode> {
    let out_dir = options
        .out_dir
        .clone()
        .or_else(|| run_dir.parent().map(Path::to_path_buf))
        .ok_or_else(|| ExitCode::from(3))?;
    if out_dir.starts_with(run_dir) {
        return Err(ExitCode::from(3));
    }
    Ok(out_dir)
}

fn collect_node_contexts(run_dir: &Path) -> Result<BTreeMap<String, RepairNodeContext>, ExitCode> {
    let mut contexts = BTreeMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(contexts);
    }
    let mut entries: Vec<_> =
        fs::read_dir(&nodes_dir).map_err(|_| ExitCode::from(3))?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace = read_typed_json::<NodeTrace>(&entry.path().join("trace.json")).ok();
        let outputs_index = read_typed_json::<bijux_dag_artifacts::OutputsIndex>(
            &entry.path().join("outputs").join("index.json"),
        )
        .ok();
        contexts.insert(node_id, RepairNodeContext { trace, outputs_index });
    }
    Ok(contexts)
}

fn collect_cache_recovery_candidates(
    manifest: Option<&Manifest>,
    node_contexts: &BTreeMap<String, RepairNodeContext>,
    issues: &[RunRepairIssue],
    cache_dir: Option<PathBuf>,
    remote_cache_dir: Option<PathBuf>,
) -> Vec<RunRepairCacheRecovery> {
    let candidate_node_ids = issues
        .iter()
        .filter(|issue| {
            matches!(
                issue.kind,
                RunRepairIssueKind::MissingOutput
                    | RunRepairIssueKind::CorruptArtifact
                    | RunRepairIssueKind::MissingOutputsIndex
                    | RunRepairIssueKind::MissingRunOutputsIndexEntry
            )
        })
        .map(|issue| issue.node_id.clone())
        .collect::<BTreeSet<_>>();
    let cache_dirs =
        resolve_cache_recovery_dirs(manifest, cache_dir.as_deref(), remote_cache_dir.as_deref());
    let mut recoveries = Vec::new();
    for node_id in candidate_node_ids {
        let Some(trace) = node_contexts.get(&node_id).and_then(|entry| entry.trace.as_ref()) else {
            continue;
        };
        let Some(cache_identity) = trace.cache_identity.as_ref() else {
            continue;
        };
        for cache_root in &cache_dirs {
            let entry = cache_root.join(&cache_identity.cache_key);
            if !entry.is_dir() {
                continue;
            }
            let verified = verify_cache_entry_cli(
                &entry,
                &cache_identity.cache_key,
                &trace.adapter_id,
                &trace.adapter_version,
            )
            .unwrap_or(false);
            if !verified {
                continue;
            }
            let Ok(index) =
                read_typed_json::<OutputsIndex>(&entry.join("outputs").join("index.json"))
            else {
                continue;
            };
            let mut restored_outputs = index
                .files
                .iter()
                .map(|file| file.name.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            restored_outputs.sort();
            let mut restored_paths =
                index.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
            restored_paths.sort();
            recoveries.push(RunRepairCacheRecovery {
                node_id: node_id.clone(),
                cache_key: cache_identity.cache_key.clone(),
                cache_entry_path: entry.display().to_string(),
                restored_outputs,
                restored_paths,
            });
            break;
        }
    }
    recoveries.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    recoveries
}

fn resolve_cache_recovery_dirs(
    manifest: Option<&Manifest>,
    cache_dir: Option<&Path>,
    remote_cache_dir: Option<&Path>,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(path) = cache_dir {
        dirs.push(path.to_path_buf());
    }
    if let Some(path) = manifest.and_then(|entry| entry.cache_dir.as_deref()).map(PathBuf::from) {
        dirs.push(path);
    }
    if let Some(path) = env_cache_dir() {
        dirs.push(path);
    }
    if let Some(path) = remote_cache_dir {
        dirs.push(path.to_path_buf());
    }
    let mut seen = BTreeSet::new();
    dirs.retain(|path| seen.insert(path.clone()));
    dirs
}

fn apply_cache_recoveries(
    run_dir: &Path,
    recoveries: &[RunRepairCacheRecovery],
) -> Result<Vec<RunRepairCacheRecovery>, ExitCode> {
    let mut applied = Vec::new();
    for recovery in recoveries {
        let cache_outputs_dir = Path::new(&recovery.cache_entry_path).join("outputs");
        let target_dir = run_dir.join("nodes").join(&recovery.node_id).join("outputs");
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).map_err(|_| ExitCode::from(3))?;
        }
        copy_dir_recursive(&cache_outputs_dir, &target_dir)?;
        applied.push(recovery.clone());
    }
    Ok(applied)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ExitCode> {
    let metadata = fs::symlink_metadata(src).map_err(|_| ExitCode::from(3))?;
    if metadata.file_type().is_symlink() {
        return Err(ExitCode::from(3));
    }
    if metadata.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|_| ExitCode::from(3))?;
        }
        fs::copy(src, dst).map_err(|_| ExitCode::from(3))?;
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|_| ExitCode::from(3))?;
    let mut entries = fs::read_dir(src)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        copy_dir_recursive(&entry.path(), &dst.join(entry.file_name()))?;
    }
    Ok(())
}

fn rebuild_run_outputs_index_from_run_dir(run_dir: &Path) -> Result<(), ExitCode> {
    let mut files = Vec::new();
    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut entries = fs::read_dir(&nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let index_path = entry.path().join("outputs").join("index.json");
            if !index_path.exists() {
                continue;
            }
            let index: OutputsIndex = read_typed_json(&index_path)?;
            for file in index.files {
                files.push(RunOutputFile {
                    node_id: file.node_id,
                    node_fingerprint: file.node_fingerprint,
                    name: file.name,
                    kind: file.kind,
                    media_type: file.media_type,
                    size_bytes: file.size_bytes,
                    sha256: file.sha256,
                    path: format!("nodes/{node_id}/outputs/{}", file.path),
                    promotable: file.promotable,
                });
            }
        }
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    write_run_outputs_index(run_dir.join("outputs"), &RunOutputsIndex { files })
        .map_err(|_| ExitCode::from(3))
}

fn read_manifest_tolerant(run_dir: &Path) -> Result<Option<Manifest>, ExitCode> {
    let manifest_path = run_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Ok(None);
    }
    Ok(read_typed_json(&manifest_path).ok())
}

fn read_manifest_status(run_dir: &Path) -> Result<String, ExitCode> {
    let manifest_path = run_dir.join("manifest.json");
    if !manifest_path.exists() {
        return Ok("unknown".to_string());
    }
    Ok(read_json_value(&manifest_path)?
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string())
}

fn read_run_snapshot(run_dir: &Path) -> Result<Option<RunSnapshot>, ExitCode> {
    let path = run_dir.join("run.snapshot.json");
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(read_typed_json(&path)?))
}

fn read_run_id_with_fallback(
    run_dir: &Path,
    run_snapshot: Option<&RunSnapshot>,
) -> Result<String, ExitCode> {
    if let Ok(run_id) = read_run_id(run_dir) {
        return Ok(run_id);
    }
    if let Some(snapshot) = run_snapshot {
        return Ok(snapshot.run_id.to_string());
    }
    Ok(run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("unknown-run")
        .trim_start_matches("run-")
        .to_string())
}

fn read_run_outputs_index_tolerant(run_dir: &Path) -> Result<Option<RunOutputsIndex>, ExitCode> {
    let path = run_dir.join("outputs").join("index.json");
    if !path.exists() {
        return Ok(None);
    }
    Ok(read_typed_json(&path).ok())
}

fn run_outputs_contains(
    run_outputs_index: &Option<RunOutputsIndex>,
    node_id: &str,
    output_name: &str,
    output_relpath: &str,
) -> bool {
    run_outputs_index.as_ref().is_some_and(|index| {
        let expected_path = format!("nodes/{node_id}/outputs/{output_relpath}");
        index.files.iter().any(|file| {
            file.node_id == node_id && file.name == output_name && file.path == expected_path
        })
    })
}

fn is_repair_failure_status(status: &str) -> bool {
    matches!(
        parse_run_state_str(status),
        Some(RunState::Failed | RunState::Cancelled | RunState::TimedOut)
    ) || matches!(status, "failed" | "cancelled" | "timed_out")
}

fn parse_run_state_str(status: &str) -> Option<RunState> {
    match status {
        "submitted" => Some(RunState::Submitted),
        "planning" => Some(RunState::Planning),
        "running" => Some(RunState::Running),
        "paused" => Some(RunState::Paused),
        "interrupted" => Some(RunState::Interrupted),
        "cancelling" => Some(RunState::Cancelling),
        "cancelled" => Some(RunState::Cancelled),
        "timed_out" => Some(RunState::TimedOut),
        "failed" => Some(RunState::Failed),
        "success" | "succeeded" => Some(RunState::Succeeded),
        _ => None,
    }
}

fn build_manifest_from_run_dir(run_dir: &Path) -> Result<Manifest, ExitCode> {
    let run_snapshot: RunSnapshot = read_typed_json(&run_dir.join("run.snapshot.json"))?;
    let graph_snapshot = read_json_value(&run_dir.join("graph.snapshot.json"))?;
    let traces = read_node_traces(run_dir)?;
    let outputs_index = if run_dir.join("outputs").join("index.json").exists() {
        Some(read_typed_json::<RunOutputsIndex>(&run_dir.join("outputs").join("index.json"))?)
    } else {
        None
    };

    let mut adapters_seen = BTreeSet::new();
    let mut adapters = Vec::new();
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut skipped = 0u32;
    let mut cached = 0u32;
    let mut cancelled = 0u32;
    let mut created_unix_ms = u128::MAX;
    let mut finished_unix_ms = 0u128;
    let mut status = "success".to_string();
    for trace in &traces {
        created_unix_ms = created_unix_ms.min(trace.started_unix_ms);
        finished_unix_ms = finished_unix_ms.max(trace.finished_unix_ms);
        match trace.status.as_str() {
            "success" => success += 1,
            "failed" => {
                failed += 1;
                if trace.lifecycle_state.as_deref() == Some("timed_out")
                    || trace.failure.as_ref().map(|failure| failure.code.as_str())
                        == Some("RUN_TIMEOUT")
                {
                    status = "timed_out".to_string();
                } else if status != "timed_out" {
                    status = "failed".to_string();
                }
            }
            "skipped" => skipped += 1,
            "cached" => cached += 1,
            "cancelled" => {
                cancelled += 1;
                status = "cancelled".to_string();
            }
            _ => {}
        }
        let key = format!("{}:{}", trace.adapter_id, trace.adapter_version);
        if adapters_seen.insert(key) {
            adapters.push(AdapterInfo {
                adapter_id: trace.adapter_id.clone(),
                adapter_version: trace.adapter_version.clone(),
                effects: Vec::new(),
            });
        }
    }
    if created_unix_ms == u128::MAX {
        created_unix_ms = 0;
    }
    let outputs = outputs_index
        .map(|index| {
            index
                .files
                .into_iter()
                .map(|file| OutputSummary {
                    node_id: file.node_id,
                    node_fingerprint: file.node_fingerprint,
                    name: file.name,
                    path: file.path,
                    kind: file.kind,
                    media_type: file.media_type,
                    size_bytes: file.size_bytes,
                    sha256: file.sha256,
                    promotable: file.promotable,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let run_cancellation_cause = if status == "cancelled" {
        let audit_path = run_dir.join("run.audit.json");
        if audit_path.exists() {
            read_json_value(&audit_path).ok().and_then(|value| {
                value.as_array().and_then(|events| {
                    events.iter().find_map(|event| {
                        (event.get("action").and_then(Value::as_str) == Some("cancel"))
                            .then_some("operator_interrupt".to_string())
                    })
                })
            })
        } else {
            None
        }
    } else {
        None
    };
    Ok(Manifest {
        manifest_version: "run-manifest/v0.1".to_string(),
        run_id: run_snapshot.run_id.to_string(),
        created_unix_ms,
        started_unix_ms: created_unix_ms,
        finished_unix_ms,
        graph_snapshot: "graph.snapshot.json".to_string(),
        status,
        spec: "bijux-dag/v0.1".to_string(),
        graph_fingerprint: graph_snapshot
            .get("graph_fingerprint")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        planner_fingerprint: None,
        execution_fingerprint: None,
        evidence_fingerprint: None,
        tool_version: "recovered-local".to_string(),
        jobs: traces.len().max(1),
        adapters,
        outputs,
        node_counts: NodeCounts { success, failed, skipped, cached, cancelled },
        policy: PolicyInfo {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: false,
            container_image_reference_policy:
                bijux_dag_artifacts::ContainerImageReferencePolicy::RequireDigest,
        },
        cache_mode: None,
        cache_dir: None,
        run_timeout_ms: None,
        run_timeout_behavior: None,
        run_cancellation_cause,
        run_metadata: Some(RunMetadata {
            submission_source: run_snapshot.submission_source,
            trigger_source: run_snapshot.trigger_source,
            operator: run_snapshot.operator,
            labels: run_snapshot.labels,
            parent_run_id: run_snapshot.parent_run_id.map(|id| id.to_string()),
            source_run_id: run_snapshot.replay_source_run_id.map(|id| id.to_string()),
            graph_inputs: BTreeMap::new(),
        }),
        run_summary: Some(RunSummary {
            total_nodes: success + failed + skipped + cached + cancelled,
            success,
            failed,
            skipped,
            cached,
            cancelled,
            promoted_outputs: Vec::new(),
        }),
    })
}

fn rebuild_run_log_index(run_dir: &Path) -> Result<Vec<Value>, ExitCode> {
    let raw = read_file(&run_dir.join("run.log.jsonl"))?;
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).map_err(|_| ExitCode::from(3)))
        .collect()
}

fn read_node_traces(run_dir: &Path) -> Result<Vec<NodeTrace>, ExitCode> {
    let mut traces = Vec::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(traces);
    }
    let mut entries: Vec<_> =
        fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))?.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let trace_path = entry.path().join("trace.json");
        if !trace_path.exists() {
            continue;
        }
        traces.push(read_typed_json(&trace_path)?);
    }
    Ok(traces)
}

fn read_typed_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn read_json_value(path: &Path) -> Result<Value, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

#[cfg(test)]
mod tests {
    use super::{plan_run_repair, RepairExecutionOptions, RunRepairIssueKind};
    use crate::ExitCode;
    use bijux_dag_runtime::{
        cache_key_explanation, CacheKeyInput, CacheMode, MaterializeMode,
        CACHE_ENTRY_MANIFEST_VERSION, CACHE_METADATA_VERSION,
    };
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn write_basic_run_snapshot(path: &Path, run_id: &str, selected_nodes: &[&str]) {
        fs::write(
            path.join("run.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": run_id,
                "graph_snapshot_path": "graph.snapshot.json",
                "planner_config": "{}",
                "scheduler_config": "{}",
                "policy_config": "{}",
                "provenance": "{}",
                "submission_source": "manual",
                "trigger_source": "cli",
                "operator": "ops",
                "labels": [],
                "parent_run_id": null,
                "requested_selectors": [],
                "selected_nodes": selected_nodes,
                "dependency_closure_enabled": true,
                "replay_source_run_id": null,
                "partial_rerun_contract": null
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
    }

    fn write_cache_recoverable_render_run(root: &Path) -> (PathBuf, String) {
        let cache_dir = root.join("cache");
        let render_dir = root.join("nodes").join("render").join("outputs").join("render");
        fs::create_dir_all(&render_dir).expect("render outputs");
        fs::create_dir_all(root.join("outputs")).expect("run outputs");
        fs::create_dir_all(&cache_dir).expect("cache dir");

        fs::write(
            root.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "spec": "bijux-dag/v0.1",
                    "nodes": [
                        {
                            "id": "render",
                            "kind": "const",
                            "outputs": [{"name":"html","path":"render/report.html","required":true}],
                            "params": {"value":"healthy-html"}
                        }
                    ],
                    "edges": []
                },
                "graph_fingerprint": "repair-fp"
            }))
            .expect("graph snapshot"),
        )
        .expect("write graph snapshot");
        write_basic_run_snapshot(root, "repair-source", &["render"]);

        let healthy_html = "<h1>healthy-html</h1>\n".to_string();
        let output_path = render_dir.join("report.html");
        fs::write(&output_path, healthy_html.as_bytes()).expect("write render output");
        let output_sha = bijux_dag_artifacts::sha256_artifact_path(&output_path).expect("sha");

        let key_input = CacheKeyInput {
            execution_fingerprint: "fp-render".to_string(),
            node_definition_fingerprint: "node-def-fp".to_string(),
            declared_environment_fingerprint: "env-fp".to_string(),
            input_lineage_fingerprint: "input-fp".to_string(),
            adapter_id: "const".to_string(),
            adapter_version: "v1".to_string(),
            output_schema_version: "schema/v1".to_string(),
            policy_fingerprint: "policy-fp".to_string(),
            execution_contract_fingerprint: "exec-fp".to_string(),
            backend_class: "local".to_string(),
        };
        let cache_key = cache_key_explanation(&key_input).key;

        fs::write(
            root.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id":"repair-source",
                "status":"success",
                "graph_fingerprint":"repair-fp",
                "cache_mode":"ReadWrite",
                "cache_dir": cache_dir.display().to_string(),
                "policy":{"deny_network":false,"deny_env":false,"deny_clock":false,"clean_env":false}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(root.join("run.log.jsonl"), "{\"event\":\"run_started\",\"ts\":1}\n")
            .expect("write log");
        fs::write(
            root.join("run-log.index.json"),
            serde_json::to_vec_pretty(&vec![json!({"event":"run_started","ts":1})]).expect("index"),
        )
        .expect("write log index");
        fs::write(
            root.join("nodes").join("render").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"render",
                "status":"success",
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "attempt":1,
                "fingerprint":"fp-render",
                "adapter_id":"const",
                "adapter_version":"v1",
                "adapter_outputs_schema_version":"schema/v1",
                "cache_identity":{
                    "cache_key": cache_key,
                    "node_definition_fingerprint":"node-def-fp",
                    "declared_environment_fingerprint":"env-fp",
                    "input_lineage_fingerprint":"input-fp",
                    "params_fingerprint":"params-fp",
                    "command_fingerprint":"cmd-fp",
                    "policy_fingerprint":"policy-fp",
                    "execution_contract_fingerprint":"exec-fp",
                    "backend_class":"local"
                }
            }))
            .expect("trace"),
        )
        .expect("write trace");
        fs::write(
            root.join("nodes").join("render").join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "files": [{
                    "name": "html",
                    "path": "render/report.html",
                    "kind": "file",
                    "media_type": "text/html",
                    "size_bytes": healthy_html.len(),
                    "sha256": output_sha,
                    "node_id": "render",
                    "node_fingerprint": "fp-render"
                }]
            }))
            .expect("node index"),
        )
        .expect("write node index");
        fs::write(
            root.join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({ "files": [] })).expect("run outputs"),
        )
        .expect("write run outputs");

        let cache_entry = cache_dir.join(&cache_key);
        fs::create_dir_all(cache_entry.join("outputs").join("render")).expect("cache outputs");
        fs::write(
            cache_entry.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "manifest_version": CACHE_ENTRY_MANIFEST_VERSION,
                "cache_key": cache_key,
                "node_id": "render",
                "outputs": [{
                    "name": "html",
                    "path": "render/report.html",
                    "kind": "file",
                    "media_type": "text/html",
                    "required": true
                }]
            }))
            .expect("cache manifest"),
        )
        .expect("write cache manifest");
        fs::write(
            cache_entry.join("meta.json"),
            serde_json::to_vec_pretty(&json!({
                "cache_metadata_version": CACHE_METADATA_VERSION,
                "cache_key": cache_key,
                "node_id": "render",
                "node_fingerprint": "fp-render",
                "node_definition_fingerprint": "node-def-fp",
                "declared_environment_fingerprint": "env-fp",
                "input_lineage_fingerprint": "input-fp",
                "params_fingerprint": "params-fp",
                "command_fingerprint": "cmd-fp",
                "adapter_id": "const",
                "adapter_version": "v1",
                "produces_outputs_schema_version": "schema/v1",
                "policy_fingerprint": "policy-fp",
                "execution_contract_fingerprint": "exec-fp",
                "backend_class": "local",
                "created_unix_ms": 1,
                "cache_source": "local",
                "schema_version": "v0.1"
            }))
            .expect("cache meta"),
        )
        .expect("write cache meta");
        fs::write(
            cache_entry.join("outputs").join("render").join("report.html"),
            healthy_html.as_bytes(),
        )
        .expect("write cache payload");
        fs::write(
            cache_entry.join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "files": [{
                    "name": "html",
                    "path": "render/report.html",
                    "kind": "file",
                    "media_type": "text/html",
                    "size_bytes": healthy_html.len(),
                    "sha256": output_sha,
                    "node_id": "render",
                    "node_fingerprint": "fp-render"
                }]
            }))
            .expect("cache index"),
        )
        .expect("write cache index");

        fs::write(&output_path, "<h1>corrupt-html</h1>\n").expect("corrupt output");
        (cache_dir, healthy_html)
    }

    #[test]
    fn plan_run_repair_detects_failed_missing_and_corrupt_damage() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("nodes").join("extract").join("outputs"))
            .expect("extract outputs");
        fs::create_dir_all(dir.path().join("nodes").join("render").join("outputs"))
            .expect("render outputs");
        fs::create_dir_all(dir.path().join("nodes").join("publish").join("outputs"))
            .expect("publish outputs");
        fs::create_dir_all(dir.path().join("outputs")).expect("outputs");

        fs::write(
            dir.path().join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "spec": "bijux-dag/v0.1",
                    "nodes": [
                        {
                            "id": "extract",
                            "kind": "const",
                            "outputs": [{"name":"raw","path":"extract/raw.json","required":true}],
                            "params": {"value":{"ok":true}}
                        },
                        {
                            "id": "render",
                            "kind": "const",
                            "inputs": ["raw"],
                            "outputs": [{"name":"html","path":"render/report.html","required":true}],
                            "params": {"value":"rendered"}
                        },
                        {
                            "id": "publish",
                            "kind": "shell",
                            "inputs": ["html"],
                            "outputs": [{"name":"bulletin","path":"publish/bulletin.md","required":true}],
                            "params": {"argv":["/bin/sh","-c","exit 1"]}
                        }
                    ],
                    "edges": [
                        {"from":{"node_id":"extract","port":"raw"},"to":{"node_id":"render","port":"raw"}},
                        {"from":{"node_id":"render","port":"html"},"to":{"node_id":"publish","port":"html"}}
                    ]
                },
                "graph_fingerprint": "repair-fp"
            }))
            .expect("graph snapshot"),
        )
        .expect("write graph snapshot");
        write_basic_run_snapshot(dir.path(), "repair-source", &["extract", "render", "publish"]);
        fs::write(
            dir.path().join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id":"repair-source",
                "status":"failed",
                "graph_fingerprint":"repair-fp",
                "policy":{"deny_network":false,"deny_env":false,"deny_clock":false,"clean_env":false}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(dir.path().join("run.log.jsonl"), "{\"event\":\"run_started\",\"ts\":1}\n")
            .expect("write log");
        fs::write(
            dir.path().join("run-log.index.json"),
            serde_json::to_vec_pretty(&vec![json!({"event":"run_started","ts":1})]).expect("index"),
        )
        .expect("write log index");

        fs::write(
            dir.path().join("nodes").join("extract").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"extract",
                "status":"success",
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "attempt":1,
                "fingerprint":"fp-extract",
                "adapter_id":"const",
                "adapter_version":"v1",
                "adapter_outputs_schema_version":"schema/v1"
            }))
            .expect("trace"),
        )
        .expect("write extract trace");
        fs::write(
            dir.path().join("nodes").join("render").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"render",
                "status":"success",
                "started_unix_ms":3,
                "finished_unix_ms":4,
                "attempt":1,
                "fingerprint":"fp-render",
                "adapter_id":"const",
                "adapter_version":"v1",
                "adapter_outputs_schema_version":"schema/v1"
            }))
            .expect("trace"),
        )
        .expect("write render trace");
        fs::write(
            dir.path().join("nodes").join("publish").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"publish",
                "status":"failed",
                "started_unix_ms":5,
                "finished_unix_ms":6,
                "attempt":1,
                "fingerprint":"fp-publish",
                "adapter_id":"shell",
                "adapter_version":"v1",
                "adapter_outputs_schema_version":"schema/v1",
                "failure":{"kind":"Execution","code":"EXIT_NONZERO","message":"failed"}
            }))
            .expect("trace"),
        )
        .expect("write publish trace");

        fs::create_dir_all(
            dir.path().join("nodes").join("extract").join("outputs").join("extract"),
        )
        .expect("extract output dir");
        fs::create_dir_all(dir.path().join("nodes").join("render").join("outputs").join("render"))
            .expect("render output dir");
        fs::write(
            dir.path()
                .join("nodes")
                .join("extract")
                .join("outputs")
                .join("extract")
                .join("raw.json"),
            b"{\"ok\":true}",
        )
        .expect("write extract output");
        fs::write(
            dir.path()
                .join("nodes")
                .join("render")
                .join("outputs")
                .join("render")
                .join("report.html"),
            b"stale-html",
        )
        .expect("write render output");
        fs::write(
            dir.path().join("nodes").join("extract").join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "files": [{
                    "name": "raw",
                    "path": "extract/raw.json",
                    "kind": "value",
                    "media_type": "application/json",
                    "size_bytes": 11,
                    "sha256": bijux_dag_artifacts::sha256_artifact_path(
                        dir.path().join("nodes").join("extract").join("outputs").join("extract").join("raw.json")
                    ).expect("sha"),
                    "node_id": "extract",
                    "node_fingerprint": "fp-extract"
                }]
            }))
            .expect("extract index"),
        )
        .expect("write extract index");
        fs::write(
            dir.path().join("nodes").join("render").join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "files": [{
                    "name": "html",
                    "path": "render/report.html",
                    "kind": "file",
                    "media_type": "text/html",
                    "size_bytes": 9,
                    "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                    "node_id": "render",
                    "node_fingerprint": "fp-render"
                }]
            }))
            .expect("render index"),
        )
        .expect("write render index");
        fs::write(
            dir.path().join("outputs").join("index.json"),
            serde_json::to_vec_pretty(&json!({
                "files": [{
                    "node_id": "extract",
                    "node_fingerprint": "fp-extract",
                    "name": "raw",
                    "kind": "value",
                    "media_type": "application/json",
                    "size_bytes": 11,
                    "sha256": bijux_dag_artifacts::sha256_artifact_path(
                        dir.path().join("nodes").join("extract").join("outputs").join("extract").join("raw.json")
                    ).expect("sha"),
                    "path": "nodes/extract/outputs/extract/raw.json"
                }]
            }))
            .expect("run outputs"),
        )
        .expect("write run outputs");

        let report = plan_run_repair(dir.path(), None, None).expect("plan repair");
        assert_eq!(report.repair_roots, vec!["render".to_string()]);
        assert!(report.invalidated_nodes.iter().any(|node_id| node_id == "publish"));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RunRepairIssueKind::FailedNode && issue.node_id == "publish"
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RunRepairIssueKind::CorruptArtifact && issue.node_id == "render"
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RunRepairIssueKind::MissingRunOutputsIndexEntry
                && issue.node_id == "render"
        }));
    }

    #[test]
    fn apply_run_repair_blocks_nested_output_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(dir.path().join("nodes").join("publish").join("outputs"))
            .expect("nodes");
        fs::create_dir_all(dir.path().join("outputs")).expect("outputs");
        fs::write(
            dir.path().join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "spec":"bijux-dag/v0.1",
                    "nodes":[
                        {
                            "id":"publish",
                            "kind":"const",
                            "outputs":[{"name":"bulletin","path":"publish/bulletin.md","required":true}],
                            "params":{"value":"ok"}
                        }
                    ],
                    "edges":[]
                },
                "graph_fingerprint": "fp"
            }))
            .expect("graph"),
        )
        .expect("write graph");
        write_basic_run_snapshot(dir.path(), "repair-source", &["publish"]);
        fs::write(
            dir.path().join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id":"repair-source",
                "status":"success",
                "graph_fingerprint":"fp",
                "policy":{"deny_network":false,"deny_env":false,"deny_clock":false,"clean_env":false}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(dir.path().join("run.log.jsonl"), "{\"event\":\"run_started\",\"ts\":1}\n")
            .expect("write log");
        fs::write(
            dir.path().join("nodes").join("publish").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"publish",
                "status":"success",
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "attempt":1,
                "fingerprint":"fp-publish",
                "adapter_id":"const",
                "adapter_version":"v1",
                "adapter_outputs_schema_version":"schema/v1"
            }))
            .expect("trace"),
        )
        .expect("write trace");
        fs::write(
            dir.path().join("run-log.index.json"),
            serde_json::to_vec_pretty(&vec![json!({"event":"run_started","ts":1})]).expect("index"),
        )
        .expect("write log index");
        fs::write(dir.path().join("outputs").join("index.json"), "{\"files\":[]}")
            .expect("run outputs");

        let err = super::apply_run_repair(
            dir.path(),
            &RepairExecutionOptions {
                out_dir: Some(dir.path().join("nested")),
                run_id: Some("repair-child".to_string()),
                jobs: 1,
                materialize_inputs: MaterializeMode::Copy,
                cache_mode: CacheMode::Off,
                cache_dir: None,
                remote_cache_dir: None,
            },
        )
        .expect_err("nested output root should fail");
        assert_eq!(err, ExitCode::from(3));
    }

    #[test]
    fn plan_run_repair_reports_exact_cache_recovery_for_corrupt_outputs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (cache_dir, _) = write_cache_recoverable_render_run(dir.path());

        let report =
            plan_run_repair(dir.path(), Some(cache_dir.clone()), None).expect("plan repair");
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RunRepairIssueKind::CorruptArtifact && issue.node_id == "render"
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == RunRepairIssueKind::MissingRunOutputsIndexEntry
                && issue.node_id == "render"
        }));
        assert!(report.repair_roots.is_empty(), "exact cache recovery should avoid rerun roots");
        assert_eq!(report.cache_recovery_candidates.len(), 1);
        assert_eq!(report.cache_recovery_candidates[0].node_id, "render");
        assert!(
            report.proposed_actions.iter().any(|action| {
                action.kind == super::RunRepairActionKind::RestoreFromCache
                    && action.node_roots == vec!["render".to_string()]
            }),
            "expected restore-from-cache action in repair plan"
        );
    }

    #[test]
    fn apply_run_repair_restores_corrupt_outputs_from_verified_cache() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (cache_dir, healthy_html) = write_cache_recoverable_render_run(dir.path());

        let report = super::apply_run_repair(
            dir.path(),
            &RepairExecutionOptions {
                out_dir: None,
                run_id: None,
                jobs: 1,
                materialize_inputs: MaterializeMode::Copy,
                cache_mode: CacheMode::Off,
                cache_dir: Some(cache_dir.clone()),
                remote_cache_dir: None,
            },
        )
        .expect("apply repair");

        assert!(report.issues.is_empty(), "cache restoration should clear output issues");
        assert!(report.repair_run.is_none(), "exact cache restoration should avoid child reruns");
        assert_eq!(report.cache_recoveries_applied.len(), 1);
        assert_eq!(report.cache_recoveries_applied[0].node_id, "render");

        let restored = fs::read_to_string(
            dir.path()
                .join("nodes")
                .join("render")
                .join("outputs")
                .join("render")
                .join("report.html"),
        )
        .expect("restored output");
        assert_eq!(restored, healthy_html);

        let run_outputs: serde_json::Value = serde_json::from_slice(
            &fs::read(dir.path().join("outputs").join("index.json")).expect("run outputs"),
        )
        .expect("run outputs json");
        assert_eq!(run_outputs["files"].as_array().map_or(0, Vec::len), 1);
        assert_eq!(run_outputs["files"][0]["path"], "nodes/render/outputs/render/report.html");
    }
}
