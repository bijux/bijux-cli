use crate::commands::DiffModeArg;
use crate::diff::{build_run_diff, RunDiff};
use bijux_dag_artifacts::lineage::ArtifactLineageSnapshot;
use bijux_dag_artifacts::{sha256_artifact_path, InputsIndex, OutputsIndex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Deserialize)]
struct GraphSnapshot {
    graph_fingerprint: String,
}

#[derive(Debug)]
pub(crate) struct RunMaterial {
    pub manifest: Value,
    pub graph_snapshot: Value,
    pub graph_fingerprint: String,
    pub node_traces: HashMap<String, Value>,
    pub node_outputs: HashMap<String, OutputsIndex>,
    pub run_outputs: OutputsIndex,
    pub provenance: Option<Value>,
    pub lineage: Option<ArtifactLineageSnapshot>,
}

pub(crate) fn load_run_material(run_dir: &Path) -> Result<RunMaterial, ExitCode> {
    let manifest = read_json(&run_dir.join("manifest.json"))?;
    let graph_snapshot = read_json(&run_dir.join("graph.snapshot.json"))?;
    let snap: GraphSnapshot =
        serde_json::from_value(graph_snapshot.clone()).map_err(|_| ExitCode::from(3))?;
    let node_traces = read_node_traces(run_dir)?;
    let node_outputs = read_outputs_indexes(run_dir)?;
    let run_outputs = read_run_outputs_index(run_dir)?;
    let provenance = if run_dir.join("provenance.json").exists() {
        Some(read_json(&run_dir.join("provenance.json"))?)
    } else {
        None
    };
    let lineage = if run_dir.join("lineage.snapshot.json").exists() {
        Some(read_typed_json::<ArtifactLineageSnapshot>(&run_dir.join("lineage.snapshot.json"))?)
    } else {
        None
    };
    Ok(RunMaterial {
        manifest,
        graph_snapshot,
        graph_fingerprint: snap.graph_fingerprint,
        node_traces,
        node_outputs,
        run_outputs,
        provenance,
        lineage,
    })
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReplayBoundaryArtifactCheck {
    pub boundary_node_id: String,
    pub source_node_id: String,
    pub source_output_name: String,
    pub source_node_fingerprint: String,
    pub recorded_sha256: String,
    pub source_output_path: String,
    pub materialized_input_path: String,
    pub verified: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReplayBoundaryVerificationReport {
    pub source_run_id: String,
    pub boundary_nodes: Vec<String>,
    pub boundary_nodes_without_upstream_artifacts: Vec<String>,
    pub verified: bool,
    pub errors: Vec<String>,
    pub checks: Vec<ReplayBoundaryArtifactCheck>,
}

pub(crate) fn verify_replay_boundary_inputs(
    run_dir: &Path,
    source_run_id: &str,
    boundary_nodes: &[String],
) -> Result<ReplayBoundaryVerificationReport, ExitCode> {
    let material = load_run_material(run_dir)?;
    let mut boundary_nodes = boundary_nodes.to_vec();
    boundary_nodes.sort();
    boundary_nodes.dedup();

    let mut boundary_nodes_without_upstream_artifacts = Vec::new();
    let mut errors = Vec::new();
    let mut checks = Vec::new();

    for boundary_node_id in &boundary_nodes {
        let inputs_index_path =
            run_dir.join("nodes").join(boundary_node_id).join("inputs").join("index.json");
        if !inputs_index_path.exists() {
            errors.push(format!(
                "boundary node {} is missing inputs/index.json in source run {}",
                boundary_node_id, source_run_id
            ));
            continue;
        }
        let inputs_index: InputsIndex = read_typed_json(&inputs_index_path)?;
        if inputs_index.files.is_empty() {
            boundary_nodes_without_upstream_artifacts.push(boundary_node_id.clone());
        }
        for input in inputs_index.files {
            let mut notes = Vec::new();
            let source_trace = material.node_traces.get(&input.source_node_id);
            let source_status =
                source_trace.and_then(|trace| trace.get("status")).and_then(Value::as_str);
            if !matches!(source_status, Some("success" | "cached")) {
                notes.push(format!(
                    "source node {} is not terminally reusable in source run {}",
                    input.source_node_id, source_run_id
                ));
            }
            let trace_fingerprint =
                source_trace.and_then(|trace| trace.get("fingerprint")).and_then(Value::as_str);
            if trace_fingerprint != Some(input.source_node_fingerprint.as_str()) {
                notes.push(format!(
                    "source node fingerprint drift detected for {}",
                    input.source_node_id
                ));
            }

            let mut source_output_path = run_dir
                .join("nodes")
                .join(&input.source_node_id)
                .join("outputs")
                .join(&input.source_output_name);
            let materialized_input_path =
                run_dir.join("nodes").join(boundary_node_id).join("inputs").join(&input.local_path);

            match material.node_outputs.get(&input.source_node_id) {
                Some(index) => {
                    let source_output =
                        index.files.iter().find(|file| file.name == input.source_output_name);
                    match source_output {
                        Some(file) => {
                            if file.node_fingerprint != input.source_node_fingerprint {
                                notes.push(format!(
                                    "output index fingerprint drift detected for {}:{}",
                                    input.source_node_id, input.source_output_name
                                ));
                            }
                            if file.sha256 != input.source_sha256 {
                                notes.push(format!(
                                    "recorded source artifact hash drift detected for {}:{}",
                                    input.source_node_id, input.source_output_name
                                ));
                            }
                            let persisted_source_path = run_dir
                                .join("nodes")
                                .join(&input.source_node_id)
                                .join("outputs")
                                .join(&file.path);
                            source_output_path.clone_from(&persisted_source_path);
                            match sha256_artifact_path(&persisted_source_path) {
                                Ok(actual_sha256) if actual_sha256 != file.sha256 => {
                                    notes.push(format!(
                                        "persisted source artifact hash mismatch for {}:{}",
                                        input.source_node_id, input.source_output_name
                                    ))
                                }
                                Ok(_) => {}
                                Err(_) => notes.push(format!(
                                    "persisted source artifact is unreadable for {}:{}",
                                    input.source_node_id, input.source_output_name
                                )),
                            }
                        }
                        None => notes.push(format!(
                            "source output {} is missing from node outputs index for {}",
                            input.source_output_name, input.source_node_id
                        )),
                    }
                }
                None => notes.push(format!(
                    "source node outputs index is missing for {}",
                    input.source_node_id
                )),
            }

            match sha256_artifact_path(&materialized_input_path) {
                Ok(actual_sha256) if actual_sha256 != input.source_sha256 => notes.push(format!(
                    "materialized input hash mismatch for {} <- {}:{}",
                    boundary_node_id, input.source_node_id, input.source_output_name
                )),
                Ok(_) => {}
                Err(_) => notes.push(format!(
                    "materialized input is unreadable for {} <- {}:{}",
                    boundary_node_id, input.source_node_id, input.source_output_name
                )),
            }

            checks.push(ReplayBoundaryArtifactCheck {
                boundary_node_id: boundary_node_id.clone(),
                source_node_id: input.source_node_id,
                source_output_name: input.source_output_name,
                source_node_fingerprint: input.source_node_fingerprint,
                recorded_sha256: input.source_sha256,
                source_output_path: source_output_path.display().to_string(),
                materialized_input_path: materialized_input_path.display().to_string(),
                verified: notes.is_empty(),
                notes,
            });
        }
    }

    let verified = errors.is_empty() && checks.iter().all(|check| check.verified);
    Ok(ReplayBoundaryVerificationReport {
        source_run_id: source_run_id.to_string(),
        boundary_nodes,
        boundary_nodes_without_upstream_artifacts,
        verified,
        errors,
        checks,
    })
}

pub(crate) fn node_rerun_diff_report(
    run_a: &Path,
    run_b: &Path,
    node_id: &str,
) -> Result<Value, ExitCode> {
    let material_a = load_run_material(run_a)?;
    let material_b = load_run_material(run_b)?;
    let diff = build_run_diff(
        material_a.manifest.clone(),
        material_b.manifest.clone(),
        material_a.graph_fingerprint.clone(),
        material_b.graph_fingerprint.clone(),
        &material_a.node_traces,
        &material_b.node_traces,
        &material_a.node_outputs,
        &material_b.node_outputs,
    );
    Ok(json!({
        "node_id": node_id,
        "summary": summary_payload(&material_a, &material_b, &diff, Some(node_id)),
        "artifact": artifact_payload(run_a, run_b, &material_a, &material_b, Some(node_id))?,
        "causal_chain": build_causal_chain(&material_a, &material_b, &diff, Some(node_id)),
    }))
}

pub(crate) fn run_diff_from_dirs(run_a: &Path, run_b: &Path) -> Result<RunDiff, ExitCode> {
    let material_a = load_run_material(run_a)?;
    let material_b = load_run_material(run_b)?;
    Ok(build_run_diff(
        material_a.manifest.clone(),
        material_b.manifest.clone(),
        material_a.graph_fingerprint,
        material_b.graph_fingerprint,
        &material_a.node_traces,
        &material_b.node_traces,
        &material_a.node_outputs,
        &material_b.node_outputs,
    ))
}

pub(crate) fn run_diff_mode_payload(
    run_a: &Path,
    run_b: &Path,
    mode: DiffModeArg,
    node: Option<&str>,
) -> Result<Value, ExitCode> {
    let material_a = load_run_material(run_a)?;
    let material_b = load_run_material(run_b)?;
    let diff = build_run_diff(
        material_a.manifest.clone(),
        material_b.manifest.clone(),
        material_a.graph_fingerprint.clone(),
        material_b.graph_fingerprint.clone(),
        &material_a.node_traces,
        &material_b.node_traces,
        &material_a.node_outputs,
        &material_b.node_outputs,
    );
    match mode {
        DiffModeArg::Summary => Ok(summary_payload(&material_a, &material_b, &diff, node)),
        DiffModeArg::Semantic => Ok(semantic_payload(&material_a, &material_b, diff, node)),
        DiffModeArg::Artifact => artifact_payload(run_a, run_b, &material_a, &material_b, node),
        DiffModeArg::Provenance => Ok(provenance_payload(&material_a, &material_b)),
        DiffModeArg::Timing => Ok(timing_payload(&material_a, &material_b, node)),
        DiffModeArg::Policy => Ok(policy_payload(&material_a, &material_b)),
        DiffModeArg::Cache => Ok(cache_payload(&material_a, &material_b, node)),
        DiffModeArg::Raw => Ok(raw_payload(&material_a, &material_b, diff, node)),
    }
}

pub(crate) fn why_rerun_payload(
    run_a: &Path,
    run_b: &Path,
    node: Option<&str>,
) -> Result<Value, ExitCode> {
    let material_a = load_run_material(run_a)?;
    let material_b = load_run_material(run_b)?;
    let diff = build_run_diff(
        material_a.manifest.clone(),
        material_b.manifest.clone(),
        material_a.graph_fingerprint.clone(),
        material_b.graph_fingerprint.clone(),
        &material_a.node_traces,
        &material_b.node_traces,
        &material_a.node_outputs,
        &material_b.node_outputs,
    );
    Ok(json!({
        "root_cause_summary": diff.replay_equivalence.reason_report.summary,
        "equivalent": diff.replay_equivalence.equivalent,
        "safety_level": diff.replay_equivalence.safety_level,
        "reasons": diff.replay_equivalence.reasons,
        "cause_groups": diff.replay_equivalence.cause_groups,
        "branch_decision_drift_nodes": diff.replay_equivalence.branch_decision_drift_nodes,
        "container_digest_drift_nodes": diff.replay_equivalence.container_digest_drift_nodes,
        "adapter_binary_drift_nodes": diff.replay_equivalence.adapter_binary_drift_nodes,
        "causal_chain": build_causal_chain(&material_a, &material_b, &diff, node),
    }))
}

pub(crate) fn replay_dry_run_plan(
    run_dir: &Path,
    out: &Path,
    snapshot: &crate::run_data::GraphSnapshot,
    source_run_id: Option<&str>,
    downstream_selection_roots: &[String],
    selected_node_ids: &[String],
    selectors_include: &[String],
    selectors_exclude: &[String],
    cache_mode: &str,
    jobs: usize,
    prove: bool,
    sandbox: bool,
) -> Result<Value, ExitCode> {
    let material = load_run_material(run_dir)?;
    let target_inside_source = out.starts_with(run_dir);
    let mut node_ids = BTreeSet::new();
    for node in &snapshot.graph.nodes {
        node_ids.insert(node.id.clone());
    }
    for node_id in material.node_traces.keys() {
        node_ids.insert(node_id.clone());
    }
    let mut planned_actions = Vec::new();
    for node_id in node_ids {
        let selected = if !downstream_selection_roots.is_empty() {
            selected_node_ids.iter().any(|selected| selected == &node_id)
        } else if selectors_include.is_empty() {
            true
        } else {
            selectors_include.iter().any(|selector| selector == &node_id)
        };
        let excluded = selectors_exclude.iter().any(|selector| selector == &node_id);
        let trace = material.node_traces.get(&node_id);
        let status = trace.and_then(|value| value.get("status")).and_then(Value::as_str);
        let evidence_complete = trace
            .and_then(|value| value.get("adapter_id"))
            .and_then(Value::as_str)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
            && material.node_outputs.contains_key(&node_id);
        let (action, reason) = if sandbox && target_inside_source {
            ("forbid", "sandbox forbids writing inside the source run directory")
        } else if excluded {
            ("skip", "node excluded from replay selector set")
        } else if !selected && !downstream_selection_roots.is_empty() {
            ("skip", "node lies outside the requested downstream rerun closure")
        } else if !selected {
            ("skip", "node excluded from replay selector set")
        } else if !downstream_selection_roots.is_empty() {
            ("reexecute", "node lies inside the requested downstream rerun closure")
        } else if cache_mode != "Off" && matches!(status, Some("cached")) {
            ("cache", "node was cached in the source run and cache reuse is enabled")
        } else if sandbox && evidence_complete && matches!(status, Some("success" | "cached")) {
            (
                "reuse",
                "sandbox can reuse source evidence for inspection without mutating the source run",
            )
        } else {
            ("reexecute", "node must be replayed under the current selector and policy set")
        };
        planned_actions.push(json!({
            "node_id": node_id,
            "action": action,
            "reason": reason,
            "source_status": status,
            "evidence_complete": evidence_complete,
        }));
    }
    Ok(json!({
        "source_run_dir": run_dir,
        "target_out_dir": out,
        "source_run_id": source_run_id,
        "parent_run_id": source_run_id,
        "sandbox_mode": if sandbox { "isolated" } else { "standard" },
        "mutates_source": false,
        "forbid_reason": if sandbox && target_inside_source {
            Some("target output directory must be outside the source run when sandbox mode is enabled")
        } else {
            None
        },
        "selectors": {
            "downstream_roots": downstream_selection_roots,
            "selected_node_ids": selected_node_ids,
            "select": selectors_include,
            "exclude": selectors_exclude,
        },
        "cache_mode": cache_mode,
        "jobs": jobs,
        "prove_requested": prove,
        "planned_actions": planned_actions,
    }))
}

pub(crate) fn replay_evidence_gaps(run_dir: &Path) -> Vec<String> {
    let mut gaps = BTreeSet::new();
    let manifest_path = run_dir.join("manifest.json");
    let graph_path = run_dir.join("graph.snapshot.json");
    if !manifest_path.exists() {
        gaps.insert("missing_manifest".to_string());
    }
    if !graph_path.exists() {
        gaps.insert("missing_graph".to_string());
    }

    let manifest: Option<Value> = read_json(&manifest_path).ok();
    if manifest.as_ref().and_then(|value| value.get("policy")).is_none() {
        gaps.insert("missing_policy".to_string());
    }

    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut trace_seen = false;
        if let Ok(entries) = fs::read_dir(&nodes_dir) {
            for entry in entries.filter_map(Result::ok) {
                let node_dir = entry.path();
                if !node_dir.is_dir() {
                    continue;
                }
                let trace_path = node_dir.join("trace.json");
                if !trace_path.exists() {
                    gaps.insert("missing_trace".to_string());
                    continue;
                }
                trace_seen = true;
                if let Ok(trace) = read_json(&trace_path) {
                    let adapter_id_missing = trace
                        .get("adapter_id")
                        .and_then(Value::as_str)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true);
                    let adapter_version_missing = trace
                        .get("adapter_version")
                        .and_then(Value::as_str)
                        .map(|value| value.trim().is_empty())
                        .unwrap_or(true);
                    if adapter_id_missing || adapter_version_missing {
                        gaps.insert("missing_adapter_identity".to_string());
                    }
                    let terminal = trace
                        .get("status")
                        .and_then(Value::as_str)
                        .map(|status| matches!(status, "success" | "cached"))
                        .unwrap_or(false);
                    if terminal {
                        let outputs_index_path = node_dir.join("outputs").join("index.json");
                        if !outputs_index_path.exists() {
                            gaps.insert("missing_artifact_hash".to_string());
                        } else if let Ok(index) =
                            read_typed_json::<OutputsIndex>(&outputs_index_path)
                        {
                            if index.files.iter().any(|file| file.sha256.trim().is_empty()) {
                                gaps.insert("missing_artifact_hash".to_string());
                            }
                        } else {
                            gaps.insert("missing_artifact_hash".to_string());
                        }
                    }
                } else {
                    gaps.insert("missing_trace".to_string());
                }
            }
        }
        if !trace_seen {
            gaps.insert("missing_trace".to_string());
        }
    } else {
        gaps.insert("missing_trace".to_string());
    }

    gaps.into_iter().collect()
}

fn summary_payload(
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    diff: &RunDiff,
    node: Option<&str>,
) -> Value {
    json!({
        "mode": "summary",
        "node": node,
        "equivalent": diff.replay_equivalence.equivalent,
        "safety_level": diff.replay_equivalence.safety_level,
        "root_cause_summary": diff.replay_equivalence.reason_report.summary,
        "reasons": diff.replay_equivalence.reasons,
        "cause_groups": diff.replay_equivalence.cause_groups,
        "branch_decision_drift_nodes": diff.replay_equivalence.branch_decision_drift_nodes,
        "container_digest_drift_nodes": diff.replay_equivalence.container_digest_drift_nodes,
        "adapter_binary_drift_nodes": diff.replay_equivalence.adapter_binary_drift_nodes,
        "evidence_fingerprint_explanation": evidence_fingerprint_explanation(material_a, material_b, node),
        "compared_dimensions": diff.replay_equivalence.reason_report.compared_dimensions,
        "mismatch_dimensions": diff.replay_equivalence.reason_report.mismatch_dimensions,
    })
}

fn build_causal_chain(
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    diff: &RunDiff,
    node: Option<&str>,
) -> Vec<Value> {
    if let Some(node_id) = node {
        let mut chain = Vec::new();
        let node_diff = diff.nodes.get(node_id);
        let output_diff = diff.outputs.get(node_id);
        let trace_a = material_a.node_traces.get(node_id);
        let trace_b = material_b.node_traces.get(node_id);
        chain.push(json!({
            "level": "node",
            "node_id": node_id,
            "summary": "node-level replay analysis",
        }));
        if let Some(node_diff) = node_diff {
            if node_diff.fp_a != node_diff.fp_b {
                chain.push(json!({
                    "level": "fingerprint",
                    "node_id": node_id,
                    "a": node_diff.fp_a,
                    "b": node_diff.fp_b,
                    "summary": "node fingerprint changed, so the replay contract requires reevaluation"
                }));
            }
            if node_diff.status_a != node_diff.status_b {
                chain.push(json!({
                    "level": "status",
                    "node_id": node_id,
                    "a": node_diff.status_a,
                    "b": node_diff.status_b,
                    "summary": "node execution status drifted between runs"
                }));
            }
            if node_diff.branch_decision_a != node_diff.branch_decision_b {
                chain.push(json!({
                    "level": "branch_decision",
                    "node_id": node_id,
                    "a": node_diff.branch_decision_a,
                    "b": node_diff.branch_decision_b,
                    "summary": "branch path selection changed, so downstream activation changed"
                }));
            }
        }
        if let Some(output_diff) = output_diff {
            chain.push(json!({
                "level": "artifacts",
                "node_id": node_id,
                "added": output_diff.added,
                "removed": output_diff.removed,
                "changed": output_diff.changed,
                "summary": "artifact payload or structure changed for this node"
            }));
        }
        let evidence_a = trace_a.and_then(|value| value.get("evidence_fingerprint")).cloned();
        let evidence_b = trace_b.and_then(|value| value.get("evidence_fingerprint")).cloned();
        if evidence_a != evidence_b {
            chain.push(json!({
                "level": "evidence",
                "node_id": node_id,
                "a": evidence_a,
                "b": evidence_b,
                "summary": "node evidence fingerprint drifted"
            }));
        }
        if chain.len() == 1 {
            chain.push(json!({
                "level": "equivalent",
                "node_id": node_id,
                "summary": "no node-scoped replay drift detected in the available evidence"
            }));
        }
        return chain;
    }

    let mut chain = Vec::new();
    if diff.graph_fingerprint.is_some() {
        chain.push(json!({
            "level": "graph",
            "summary": "graph fingerprint changed, so replay equivalence is forbidden"
        }));
    }
    if !diff.manifest.is_empty() {
        chain.push(json!({
            "level": "manifest",
            "fields": diff.manifest.keys().cloned().collect::<Vec<_>>(),
            "summary": "run manifest fields changed"
        }));
    }
    if !diff.replay_equivalence.branch_decision_drift_nodes.is_empty() {
        chain.push(json!({
            "level": "branch_decisions",
            "nodes": diff.replay_equivalence.branch_decision_drift_nodes,
            "summary": "branch decisions changed for one or more nodes"
        }));
    }
    if !diff.replay_equivalence.container_digest_drift_nodes.is_empty() {
        chain.push(json!({
            "level": "container_digest",
            "nodes": diff.replay_equivalence.container_digest_drift_nodes,
            "summary": "container image digests changed for one or more nodes"
        }));
    }
    if !diff.replay_equivalence.adapter_binary_drift_nodes.is_empty() {
        chain.push(json!({
            "level": "adapter_binary",
            "nodes": diff.replay_equivalence.adapter_binary_drift_nodes,
            "summary": "adapter binary hashes changed for one or more nodes"
        }));
    }
    if !diff.outputs.is_empty() {
        chain.push(json!({
            "level": "artifacts",
            "nodes": diff.outputs.keys().cloned().collect::<Vec<_>>(),
            "summary": "artifact payload drift was detected"
        }));
    }
    if chain.is_empty() {
        chain.push(json!({
            "level": "equivalent",
            "summary": "no replay drift detected in the available evidence"
        }));
    }
    chain
}

fn semantic_payload(
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    mut diff: RunDiff,
    node: Option<&str>,
) -> Value {
    if let Some(node_id) = node {
        diff.nodes.retain(|id, _| id == node_id);
        diff.outputs.retain(|id, _| id == node_id);
        diff.replay_equivalence.branch_decision_drift_nodes.retain(|id| id == node_id);
        diff.replay_equivalence.container_digest_drift_nodes.retain(|id| id == node_id);
        diff.replay_equivalence.adapter_binary_drift_nodes.retain(|id| id == node_id);
        if let Some(count) = diff.replay_equivalence.cause_groups.get_mut("node_outcomes") {
            *count = diff.nodes.len();
        }
        if let Some(count) = diff.replay_equivalence.cause_groups.get_mut("artifact_payload") {
            *count = diff.outputs.len();
        }
    }
    let node_focus = node.map(|node_id| {
        json!({
            "node_id": node_id,
            "trace_a": material_a.node_traces.get(node_id),
            "trace_b": material_b.node_traces.get(node_id),
            "outputs_a": material_a.node_outputs.get(node_id),
            "outputs_b": material_b.node_outputs.get(node_id),
        })
    });
    let mut payload = serde_json::to_value(&diff).unwrap_or(Value::Null);
    if let Some(node_focus) = node_focus {
        payload["node_focus"] = node_focus;
    }
    payload
}

fn artifact_payload(
    run_a: &Path,
    run_b: &Path,
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    node: Option<&str>,
) -> Result<Value, ExitCode> {
    let mut paths = BTreeSet::new();
    for file in &material_a.run_outputs.files {
        if node.map(|node_id| node_id == file.node_id).unwrap_or(true) {
            paths.insert((file.node_id.clone(), file.path.clone()));
        }
    }
    for file in &material_b.run_outputs.files {
        if node.map(|node_id| node_id == file.node_id).unwrap_or(true) {
            paths.insert((file.node_id.clone(), file.path.clone()));
        }
    }
    let mut artifacts = Vec::new();
    for (node_id, path) in paths {
        let file_a = material_a
            .run_outputs
            .files
            .iter()
            .find(|file| file.node_id == node_id && file.path == path);
        let file_b = material_b
            .run_outputs
            .files
            .iter()
            .find(|file| file.node_id == node_id && file.path == path);
        let payload_a = run_a.join(&path);
        let payload_b = run_b.join(&path);
        let exists_a = payload_a.exists();
        let exists_b = payload_b.exists();
        let bytes_a = if exists_a {
            Some(fs::read(&payload_a).map_err(|_| ExitCode::from(3))?)
        } else {
            None
        };
        let bytes_b = if exists_b {
            Some(fs::read(&payload_b).map_err(|_| ExitCode::from(3))?)
        } else {
            None
        };
        let status = match (file_a, file_b) {
            (Some(_), Some(_)) if file_a.map(|f| &f.sha256) == file_b.map(|f| &f.sha256) => {
                "unchanged"
            }
            (Some(_), Some(_)) => "changed",
            (Some(_), None) => "removed",
            (None, Some(_)) => "added",
            (None, None) => "missing",
        };
        let content_diff = match (&bytes_a, &bytes_b) {
            (Some(a), Some(b)) if a != b => Some(structured_content_diff(a, b)),
            _ => None,
        };
        artifacts.push(json!({
            "node_id": node_id,
            "path": path,
            "status": status,
            "sha256_a": file_a.map(|file| file.sha256.clone()),
            "sha256_b": file_b.map(|file| file.sha256.clone()),
            "size_bytes_a": bytes_a.as_ref().map(|bytes| bytes.len() as u64),
            "size_bytes_b": bytes_b.as_ref().map(|bytes| bytes.len() as u64),
            "content_diff": content_diff,
        }));
    }
    Ok(json!({
        "mode": "artifact",
        "node": node,
        "artifacts": artifacts,
    }))
}

fn provenance_payload(material_a: &RunMaterial, material_b: &RunMaterial) -> Value {
    let provenance_a = material_a.provenance.clone().unwrap_or(Value::Null);
    let provenance_b = material_b.provenance.clone().unwrap_or(Value::Null);
    json!({
        "mode": "provenance",
        "equal": provenance_a == provenance_b,
        "diff": value_delta(&provenance_a, &provenance_b),
        "run_a": provenance_a,
        "run_b": provenance_b,
    })
}

fn timing_payload(material_a: &RunMaterial, material_b: &RunMaterial, node: Option<&str>) -> Value {
    let mut node_ids = BTreeSet::new();
    for node_id in material_a.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    for node_id in material_b.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    let nodes = node_ids
        .into_iter()
        .map(|node_id| {
            let trace_a = material_a.node_traces.get(&node_id);
            let trace_b = material_b.node_traces.get(&node_id);
            let started_a =
                trace_a.and_then(|trace| trace.get("started_unix_ms")).and_then(Value::as_u64);
            let started_b =
                trace_b.and_then(|trace| trace.get("started_unix_ms")).and_then(Value::as_u64);
            let finished_a =
                trace_a.and_then(|trace| trace.get("finished_unix_ms")).and_then(Value::as_u64);
            let finished_b =
                trace_b.and_then(|trace| trace.get("finished_unix_ms")).and_then(Value::as_u64);
            json!({
                "node_id": node_id,
                "started_unix_ms_a": started_a,
                "started_unix_ms_b": started_b,
                "finished_unix_ms_a": finished_a,
                "finished_unix_ms_b": finished_b,
                "execution_ms_a": duration_ms(started_a, finished_a),
                "execution_ms_b": duration_ms(started_b, finished_b),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "mode": "timing",
        "run_started_unix_ms_a": material_a.manifest.get("started_unix_ms"),
        "run_started_unix_ms_b": material_b.manifest.get("started_unix_ms"),
        "run_finished_unix_ms_a": material_a.manifest.get("finished_unix_ms"),
        "run_finished_unix_ms_b": material_b.manifest.get("finished_unix_ms"),
        "nodes": nodes,
    })
}

fn policy_payload(material_a: &RunMaterial, material_b: &RunMaterial) -> Value {
    let policy_a = material_a.manifest.get("policy").cloned().unwrap_or(Value::Null);
    let policy_b = material_b.manifest.get("policy").cloned().unwrap_or(Value::Null);
    json!({
        "mode": "policy",
        "equal": policy_a == policy_b,
        "diff": value_delta(&policy_a, &policy_b),
        "run_a": policy_a,
        "run_b": policy_b,
    })
}

fn cache_payload(material_a: &RunMaterial, material_b: &RunMaterial, node: Option<&str>) -> Value {
    let mut node_ids = BTreeSet::new();
    for node_id in material_a.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    for node_id in material_b.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    let nodes = node_ids
        .into_iter()
        .map(|node_id| {
            let trace_a = material_a.node_traces.get(&node_id);
            let trace_b = material_b.node_traces.get(&node_id);
            json!({
                "node_id": node_id,
                "cache_proof_a": trace_a.and_then(|trace| trace.get("cache_proof")).cloned(),
                "cache_proof_b": trace_b.and_then(|trace| trace.get("cache_proof")).cloned(),
                "adapter_schema_a": trace_a.and_then(|trace| trace.get("adapter_outputs_schema_version")).cloned(),
                "adapter_schema_b": trace_b.and_then(|trace| trace.get("adapter_outputs_schema_version")).cloned(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "mode": "cache",
        "nodes": nodes,
    })
}

fn raw_payload(
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    diff: RunDiff,
    node: Option<&str>,
) -> Value {
    json!({
        "mode": "raw",
        "node": node,
        "semantic": semantic_payload(material_a, material_b, diff, node),
        "provenance": provenance_payload(material_a, material_b),
        "policy": policy_payload(material_a, material_b),
        "timing": timing_payload(material_a, material_b, node),
        "cache": cache_payload(material_a, material_b, node),
    })
}

fn evidence_fingerprint_explanation(
    material_a: &RunMaterial,
    material_b: &RunMaterial,
    node: Option<&str>,
) -> Vec<Value> {
    let mut explanations = Vec::new();
    let manifest_a = material_a.manifest.get("evidence_fingerprint").cloned();
    let manifest_b = material_b.manifest.get("evidence_fingerprint").cloned();
    if manifest_a != manifest_b {
        explanations.push(json!({
            "scope": "run_manifest",
            "a": manifest_a,
            "b": manifest_b,
            "reason": "manifest evidence fingerprint differs",
        }));
    }
    let prov_a =
        material_a.provenance.as_ref().and_then(|value| value.get("evidence_fingerprint")).cloned();
    let prov_b =
        material_b.provenance.as_ref().and_then(|value| value.get("evidence_fingerprint")).cloned();
    if prov_a != prov_b {
        explanations.push(json!({
            "scope": "provenance",
            "a": prov_a,
            "b": prov_b,
            "reason": "provenance evidence fingerprint differs",
        }));
    }
    let mut node_ids = BTreeSet::new();
    for node_id in material_a.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    for node_id in material_b.node_traces.keys() {
        if node.map(|selected| selected == node_id).unwrap_or(true) {
            node_ids.insert(node_id.clone());
        }
    }
    for node_id in node_ids {
        let fp_a = material_a
            .node_traces
            .get(&node_id)
            .and_then(|value| value.get("evidence_fingerprint"))
            .cloned();
        let fp_b = material_b
            .node_traces
            .get(&node_id)
            .and_then(|value| value.get("evidence_fingerprint"))
            .cloned();
        if fp_a != fp_b {
            explanations.push(json!({
                "scope": "node_trace",
                "node_id": node_id,
                "a": fp_a,
                "b": fp_b,
                "reason": "node trace evidence fingerprint differs",
            }));
        }
    }
    explanations
}

fn structured_content_diff(bytes_a: &[u8], bytes_b: &[u8]) -> Value {
    if let (Ok(text_a), Ok(text_b)) = (std::str::from_utf8(bytes_a), std::str::from_utf8(bytes_b)) {
        if let (Ok(json_a), Ok(json_b)) =
            (serde_json::from_str::<Value>(text_a), serde_json::from_str::<Value>(text_b))
        {
            return json!({
                "kind": "json",
                "changes": json_change_paths("$", &json_a, &json_b),
            });
        }
        let lines_a = text_a.lines().collect::<Vec<_>>();
        let lines_b = text_b.lines().collect::<Vec<_>>();
        let max_len = lines_a.len().max(lines_b.len());
        let mut changed_lines = Vec::new();
        for index in 0..max_len {
            let line_a = lines_a.get(index).copied();
            let line_b = lines_b.get(index).copied();
            if line_a != line_b {
                changed_lines.push(json!({
                    "line": index + 1,
                    "a": line_a,
                    "b": line_b,
                }));
            }
        }
        return json!({
            "kind": "text",
            "line_count_a": lines_a.len(),
            "line_count_b": lines_b.len(),
            "changed_lines": changed_lines,
        });
    }
    json!({
        "kind": "binary",
        "size_bytes_a": bytes_a.len(),
        "size_bytes_b": bytes_b.len(),
    })
}

fn json_change_paths(prefix: &str, a: &Value, b: &Value) -> Vec<String> {
    if a == b {
        return Vec::new();
    }
    match (a, b) {
        (Value::Object(map_a), Value::Object(map_b)) => {
            let mut keys = BTreeSet::new();
            for key in map_a.keys() {
                keys.insert(key.clone());
            }
            for key in map_b.keys() {
                keys.insert(key.clone());
            }
            let mut paths = Vec::new();
            for key in keys {
                let next = format!("{prefix}.{key}");
                match (map_a.get(&key), map_b.get(&key)) {
                    (Some(left), Some(right)) => {
                        paths.extend(json_change_paths(&next, left, right))
                    }
                    _ => paths.push(next),
                }
            }
            paths
        }
        (Value::Array(items_a), Value::Array(items_b)) => {
            let max_len = items_a.len().max(items_b.len());
            let mut paths = Vec::new();
            for index in 0..max_len {
                let next = format!("{prefix}[{index}]");
                match (items_a.get(index), items_b.get(index)) {
                    (Some(left), Some(right)) => {
                        paths.extend(json_change_paths(&next, left, right))
                    }
                    _ => paths.push(next),
                }
            }
            paths
        }
        _ => vec![prefix.to_string()],
    }
}

fn duration_ms(started: Option<u64>, finished: Option<u64>) -> Option<u64> {
    match (started, finished) {
        (Some(started), Some(finished)) if finished >= started => Some(finished - started),
        _ => None,
    }
}

fn value_delta(a: &Value, b: &Value) -> Value {
    if a == b {
        json!({"equal": true})
    } else {
        json!({"equal": false, "a": a, "b": b})
    }
}

fn read_json(path: &Path) -> Result<Value, ExitCode> {
    let payload = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&payload).map_err(|_| ExitCode::from(3))
}

fn read_typed_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ExitCode> {
    let payload = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&payload).map_err(|_| ExitCode::from(3))
}

fn read_node_traces(run_dir: &Path) -> Result<HashMap<String, Value>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(map);
    }
    let mut entries: Vec<_> = fs::read_dir(nodes_dir)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        if trace_path.exists() {
            map.insert(node_id, read_json(&trace_path)?);
        }
    }
    Ok(map)
}

fn read_outputs_indexes(run_dir: &Path) -> Result<HashMap<String, OutputsIndex>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(map);
    }
    let mut entries: Vec<_> = fs::read_dir(nodes_dir)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let index_path = entry.path().join("outputs").join("index.json");
        if index_path.exists() {
            map.insert(node_id, read_typed_json(&index_path)?);
        }
    }
    Ok(map)
}

fn read_run_outputs_index(run_dir: &Path) -> Result<OutputsIndex, ExitCode> {
    let index_path = run_dir.join("outputs").join("index.json");
    if !index_path.exists() {
        return Ok(OutputsIndex { files: Vec::new() });
    }
    read_typed_json(&index_path)
}

#[cfg(test)]
mod tests {
    use super::{
        replay_dry_run_plan, replay_evidence_gaps, run_diff_from_dirs, run_diff_mode_payload,
        why_rerun_payload,
    };
    use crate::commands::DiffModeArg;
    use crate::run_data::GraphSnapshot;
    use crate::Graph;
    use serde_json::json;
    use std::fs;

    fn write(path: &std::path::Path, value: &str) {
        fs::write(path, value).expect("write test file");
    }

    #[test]
    fn replay_service_marks_identical_runs_equivalent() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("outputs")).expect("create run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("create run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
    }

    #[test]
    fn replay_service_reports_graph_mismatch() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("outputs")).expect("create run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("create run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-2"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);

        let diff = run_diff_from_dirs(&run_a, &run_b).expect("build run diff");
        assert!(!diff.replay_equivalence.equivalent);
        assert!(diff
            .replay_equivalence
            .reasons
            .iter()
            .any(|reason| { reason.contains("graph fingerprint differs") }));
    }

    #[test]
    fn replay_service_supports_node_focused_semantic_payloads() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("nodes/decide/outputs")).expect("run-a node");
        fs::create_dir_all(run_b.join("nodes/decide/outputs")).expect("run-b node");
        fs::create_dir_all(run_a.join("outputs")).expect("run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);
        write(
            &run_a.join("nodes/decide/trace.json"),
            r#"{"status":"success","fingerprint":"fp","branch_decision":"left"}"#,
        );
        write(
            &run_b.join("nodes/decide/trace.json"),
            r#"{"status":"success","fingerprint":"fp","branch_decision":"right"}"#,
        );

        let payload = run_diff_mode_payload(&run_a, &run_b, DiffModeArg::Semantic, Some("decide"))
            .expect("semantic payload");
        assert_eq!(payload["node_focus"]["node_id"], "decide");
        assert!(payload["nodes"]["decide"].is_object());
    }

    #[test]
    fn replay_service_artifact_mode_reports_json_content_changes() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("nodes/n1/outputs")).expect("run-a node");
        fs::create_dir_all(run_b.join("nodes/n1/outputs")).expect("run-b node");
        fs::create_dir_all(run_a.join("outputs")).expect("run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(
            &run_a.join("outputs/index.json"),
            r#"{"files":[{"node_id":"n1","node_fingerprint":"fp1","name":"report","kind":"file","media_type":"application/json","size_bytes":13,"sha256":"a","path":"nodes/n1/outputs/report.json"}]}"#,
        );
        write(
            &run_b.join("outputs/index.json"),
            r#"{"files":[{"node_id":"n1","node_fingerprint":"fp1","name":"report","kind":"file","media_type":"application/json","size_bytes":13,"sha256":"b","path":"nodes/n1/outputs/report.json"}]}"#,
        );
        write(
            &run_a.join("nodes/n1/outputs/index.json"),
            r#"{"files":[{"node_id":"n1","node_fingerprint":"fp1","name":"report","kind":"file","media_type":"application/json","size_bytes":13,"sha256":"a","path":"nodes/n1/outputs/report.json"}]}"#,
        );
        write(
            &run_b.join("nodes/n1/outputs/index.json"),
            r#"{"files":[{"node_id":"n1","node_fingerprint":"fp1","name":"report","kind":"file","media_type":"application/json","size_bytes":13,"sha256":"b","path":"nodes/n1/outputs/report.json"}]}"#,
        );
        write(&run_a.join("nodes/n1/outputs/report.json"), r#"{"a":1,"b":2}"#);
        write(&run_b.join("nodes/n1/outputs/report.json"), r#"{"a":1,"b":3}"#);

        let payload = run_diff_mode_payload(&run_a, &run_b, DiffModeArg::Artifact, Some("n1"))
            .expect("artifact payload");
        assert_eq!(payload["artifacts"][0]["status"], "changed");
        assert_eq!(payload["artifacts"][0]["content_diff"]["kind"], "json");
    }

    #[test]
    fn replay_service_provenance_mode_reports_drift() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("outputs")).expect("run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);
        write(
            &run_a.join("provenance.json"),
            r#"{"tool_version":"1","evidence_fingerprint":"fp-a"}"#,
        );
        write(
            &run_b.join("provenance.json"),
            r#"{"tool_version":"2","evidence_fingerprint":"fp-b"}"#,
        );

        let payload = run_diff_mode_payload(&run_a, &run_b, DiffModeArg::Provenance, None)
            .expect("provenance payload");
        assert_eq!(payload["equal"], false);
        assert_eq!(payload["run_a"]["tool_version"], "1");
        assert_eq!(payload["run_b"]["tool_version"], "2");
    }

    #[test]
    fn replay_service_summary_reports_evidence_fingerprint_explanations() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("nodes/n1")).expect("run-a node");
        fs::create_dir_all(run_b.join("nodes/n1")).expect("run-b node");
        fs::create_dir_all(run_a.join("outputs")).expect("run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("run-b outputs");
        write(
            &run_a.join("manifest.json"),
            r#"{"status":"completed","policy":{},"evidence_fingerprint":"run-a"}"#,
        );
        write(
            &run_b.join("manifest.json"),
            r#"{"status":"completed","policy":{},"evidence_fingerprint":"run-b"}"#,
        );
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_a.join("nodes/n1/trace.json"), r#"{"evidence_fingerprint":"node-a"}"#);
        write(&run_b.join("nodes/n1/trace.json"), r#"{"evidence_fingerprint":"node-b"}"#);

        let payload = run_diff_mode_payload(&run_a, &run_b, DiffModeArg::Summary, None)
            .expect("summary payload");
        assert!(payload["evidence_fingerprint_explanation"].as_array().is_some());
        assert!(!payload["evidence_fingerprint_explanation"].as_array().unwrap().is_empty());
    }

    #[test]
    fn replay_service_classifies_missing_evidence_categories() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run = tmp.path().join("run-gap");
        fs::create_dir_all(run.join("nodes/n1")).expect("create nodes");
        write(&run.join("manifest.json"), r#"{"status":"completed"}"#);
        write(
            &run.join("nodes/n1/trace.json"),
            r#"{"status":"success","adapter_id":"","adapter_version":"1"}"#,
        );

        let gaps = replay_evidence_gaps(&run);
        assert!(gaps.contains(&"missing_graph".to_string()));
        assert!(gaps.contains(&"missing_policy".to_string()));
        assert!(gaps.contains(&"missing_adapter_identity".to_string()));
        assert!(gaps.contains(&"missing_artifact_hash".to_string()));
    }

    #[test]
    fn why_rerun_payload_supports_node_scoped_causal_chain() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_a = tmp.path().join("run-a");
        let run_b = tmp.path().join("run-b");
        fs::create_dir_all(run_a.join("nodes/branch/outputs")).expect("run-a node");
        fs::create_dir_all(run_b.join("nodes/branch/outputs")).expect("run-b node");
        fs::create_dir_all(run_a.join("outputs")).expect("run-a outputs");
        fs::create_dir_all(run_b.join("outputs")).expect("run-b outputs");
        write(&run_a.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_b.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_a.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_b.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_a.join("outputs/index.json"), r#"{"files":[]}"#);
        write(&run_b.join("outputs/index.json"), r#"{"files":[]}"#);
        write(
            &run_a.join("nodes/branch/trace.json"),
            r#"{"status":"success","fingerprint":"fp-1","branch_decision":"left","evidence_fingerprint":"e1"}"#,
        );
        write(
            &run_b.join("nodes/branch/trace.json"),
            r#"{"status":"success","fingerprint":"fp-2","branch_decision":"right","evidence_fingerprint":"e2"}"#,
        );

        let payload = why_rerun_payload(&run_a, &run_b, Some("branch")).expect("why-rerun payload");
        assert_eq!(payload["equivalent"], false);
        assert!(payload["causal_chain"].as_array().is_some_and(|items| items.len() >= 2));
    }

    #[test]
    fn replay_dry_run_plan_marks_forbidden_targets_in_sandbox_mode() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_dir = tmp.path().join("source");
        fs::create_dir_all(run_dir.join("nodes/a/outputs")).expect("run node");
        fs::create_dir_all(run_dir.join("outputs")).expect("run outputs");
        write(&run_dir.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_dir.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_dir.join("outputs/index.json"), r#"{"files":[]}"#);
        write(
            &run_dir.join("nodes/a/trace.json"),
            r#"{"status":"success","adapter_id":"shell","evidence_fingerprint":"e1"}"#,
        );
        write(
            &run_dir.join("nodes/a/outputs/index.json"),
            r#"{"files":[{"node_id":"a","node_fingerprint":"fp-a","name":"out","kind":"file","media_type":"application/octet-stream","size_bytes":1,"sha256":"a","path":"nodes/a/outputs/out"}]}"#,
        );
        let snapshot = GraphSnapshot {
            graph: serde_json::from_value::<Graph>(json!({
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"g","owners":[],"tags":[]},
                "nodes":[{"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{}}],
                "edges":[]
            }))
            .expect("graph"),
            graph_fingerprint: "fp-1".to_string(),
            source_graph: None,
            source_graph_fingerprint: None,
            dynamic_expansions: Vec::new(),
        };
        let plan = replay_dry_run_plan(
            &run_dir,
            &run_dir.join("nested-target"),
            &snapshot,
            Some("source-run"),
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            &Vec::new(),
            "Read",
            1,
            false,
            true,
        )
        .expect("dry run plan");
        assert_eq!(plan["sandbox_mode"], "isolated");
        assert_eq!(plan["planned_actions"][0]["action"], "forbid");
    }

    #[test]
    fn replay_dry_run_plan_reexecutes_requested_downstream_closure() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let run_dir = tmp.path().join("source");
        fs::create_dir_all(run_dir.join("nodes/source/outputs")).expect("source outputs");
        fs::create_dir_all(run_dir.join("nodes/branch/outputs")).expect("branch outputs");
        fs::create_dir_all(run_dir.join("nodes/sink/outputs")).expect("sink outputs");
        fs::create_dir_all(run_dir.join("outputs")).expect("run outputs");
        write(&run_dir.join("manifest.json"), r#"{"status":"completed","policy":{}}"#);
        write(&run_dir.join("graph.snapshot.json"), r#"{"graph_fingerprint":"fp-1"}"#);
        write(&run_dir.join("outputs/index.json"), r#"{"files":[]}"#);
        write(
            &run_dir.join("nodes/source/trace.json"),
            r#"{"status":"success","adapter_id":"const","evidence_fingerprint":"e-source"}"#,
        );
        write(
            &run_dir.join("nodes/source/outputs/index.json"),
            r#"{"files":[{"node_id":"source","node_fingerprint":"fp-source","name":"out","kind":"file","media_type":"application/octet-stream","size_bytes":1,"sha256":"1","path":"nodes/source/outputs/out"}]}"#,
        );
        write(
            &run_dir.join("nodes/branch/trace.json"),
            r#"{"status":"cached","adapter_id":"const","evidence_fingerprint":"e-branch"}"#,
        );
        write(
            &run_dir.join("nodes/branch/outputs/index.json"),
            r#"{"files":[{"node_id":"branch","node_fingerprint":"fp-branch","name":"out","kind":"file","media_type":"application/octet-stream","size_bytes":1,"sha256":"2","path":"nodes/branch/outputs/out"}]}"#,
        );
        write(
            &run_dir.join("nodes/sink/trace.json"),
            r#"{"status":"success","adapter_id":"const","evidence_fingerprint":"e-sink"}"#,
        );
        write(
            &run_dir.join("nodes/sink/outputs/index.json"),
            r#"{"files":[{"node_id":"sink","node_fingerprint":"fp-sink","name":"out","kind":"file","media_type":"application/octet-stream","size_bytes":1,"sha256":"3","path":"nodes/sink/outputs/out"}]}"#,
        );
        let snapshot = GraphSnapshot {
            graph: serde_json::from_value::<Graph>(json!({
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"g","owners":[],"tags":[]},
                "nodes":[
                    {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out"}],"params":{}},
                    {"id":"branch","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"branch/out"}],"params":{}},
                    {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{}}
                ],
                "edges":[
                    {"from":{"node_id":"source","port":"out"},"to":{"node_id":"branch","port":"in"}},
                    {"from":{"node_id":"branch","port":"out"},"to":{"node_id":"sink","port":"in"}}
                ]
            }))
            .expect("graph"),
            graph_fingerprint: "fp-1".to_string(),
            source_graph: None,
            source_graph_fingerprint: None,
            dynamic_expansions: Vec::new(),
        };
        let plan = replay_dry_run_plan(
            &run_dir,
            &tmp.path().join("replay"),
            &snapshot,
            Some("source-run"),
            &["branch".to_string()],
            &["branch".to_string(), "sink".to_string()],
            &Vec::new(),
            &Vec::new(),
            "Read",
            1,
            false,
            false,
        )
        .expect("dry run plan");
        assert_eq!(plan["selectors"]["downstream_roots"], serde_json::json!(["branch"]));
        assert_eq!(plan["selectors"]["selected_node_ids"], serde_json::json!(["branch", "sink"]));
        let actions = plan["planned_actions"].as_array().expect("planned actions");
        assert_eq!(
            actions.iter().find(|entry| entry["node_id"] == "source").expect("source action")
                ["action"],
            "skip"
        );
        assert_eq!(
            actions.iter().find(|entry| entry["node_id"] == "branch").expect("branch action")
                ["action"],
            "reexecute"
        );
        assert_eq!(
            actions.iter().find(|entry| entry["node_id"] == "sink").expect("sink action")["action"],
            "reexecute"
        );
    }
}
