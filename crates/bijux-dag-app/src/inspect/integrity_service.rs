use crate::run_data::load_snapshot;
use crate::{read_file, ExitCode};
use bijux_dag_artifacts::{
    build_artifact_identity, ArtifactIdentity, Manifest, RunDirSchemaIndex, RunOutputFile,
    RunOutputsIndex,
};
use bijux_dag_core::Effect;
use bijux_dag_runtime::{
    reconstruct_timeline_from_events, run_summary_invariant_ok, terminal_run_has_terminal_node,
    trace_time_order_ok, verify_event_log_completeness, EventRecord, NodeStatus, RunNodeCounts,
    TimelineExport,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn timeline_terminal_event_name(status: &str) -> &'static str {
    match status {
        "skipped" | "cancelled" => "node_skipped",
        _ => "node_finished",
    }
}

pub(crate) fn hash_run_dir(run_dir: &Path) -> Result<String, ExitCode> {
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

fn legacy_artifact_id(file: &RunOutputFile) -> String {
    format!(
        "{}:{}",
        file.node_id,
        Path::new(&file.path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(file.path.as_str())
    )
}

fn lineage_lookup_ids(file: &RunOutputFile) -> Vec<String> {
    let mut ids = vec![legacy_artifact_id(file)];
    let declared_output_id = format!("{}:{}", file.node_id, file.name);
    if !ids.iter().any(|candidate| candidate == &declared_output_id) {
        ids.push(declared_output_id);
    }
    ids
}

fn output_name(file: &RunOutputFile) -> String {
    Path::new(&file.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(file.path.as_str())
        .to_string()
}

fn artifact_identity(run_id: &str, file: &RunOutputFile) -> ArtifactIdentity {
    build_artifact_identity(run_id, &file.node_id, &file.path, &file.node_fingerprint, &file.sha256)
}

fn find_output_by_artifact_id<'a>(
    run_outputs: &'a RunOutputsIndex,
    run_id: &str,
    artifact_id: &str,
) -> Result<(&'a RunOutputFile, ArtifactIdentity, String), ExitCode> {
    run_outputs
        .files
        .iter()
        .find_map(|file| {
            let canonical = artifact_identity(run_id, file);
            let legacy = legacy_artifact_id(file);
            (artifact_id == legacy || artifact_id == canonical.canonical_artifact_id)
                .then_some((file, canonical, legacy))
        })
        .ok_or_else(|| ExitCode::from(3))
}

fn read_typed_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn is_external_artifact_id(artifact_id: &str) -> bool {
    artifact_id.starts_with("source:")
        || artifact_id.starts_with("external:")
        || artifact_id.contains("://")
}

fn is_known_wildcard_upstream(artifact_id: &str, known_node_ids: &BTreeSet<String>) -> bool {
    artifact_id.strip_suffix(":*").map(|node_id| known_node_ids.contains(node_id)).unwrap_or(false)
}

pub fn inspect_artifact(run_dir: &Path, artifact_id: &str) -> Result<Value, ExitCode> {
    let manifest: Manifest = read_typed_json(&run_dir.join("manifest.json"))?;
    let run_outputs: RunOutputsIndex =
        read_typed_json(&run_dir.join("outputs").join("index.json"))?;
    let (output, canonical_identity, legacy_id) =
        find_output_by_artifact_id(&run_outputs, &manifest.run_id, artifact_id)?;
    let artifact_path = run_dir.join(&output.path);
    let (size_bytes, payload_missing) = match fs::metadata(&artifact_path) {
        Ok(_) => (Some(output.size_bytes), false),
        Err(_) => (None, true),
    };
    let lineage_path = run_dir.join("lineage.snapshot.json");
    let lineage = if lineage_path.exists() {
        let snapshot: bijux_dag_artifacts::lineage::ArtifactLineageSnapshot =
            read_typed_json(&lineage_path)?;
        let lookup_ids = lineage_lookup_ids(output);
        let upstream = lookup_ids
            .iter()
            .flat_map(|candidate| {
                bijux_dag_artifacts::platform::lineage_dependencies(&snapshot, candidate)
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let downstream = lookup_ids
            .iter()
            .flat_map(|candidate| {
                bijux_dag_artifacts::platform::lineage_dependents(&snapshot, candidate)
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        json!({
            "subject_artifact_id": canonical_identity.canonical_artifact_id,
            "subject_legacy_artifact_id": legacy_id,
            "upstream_artifact_ids": upstream,
            "downstream_artifact_ids": downstream
        })
    } else {
        json!({
            "subject_artifact_id": canonical_identity.canonical_artifact_id,
            "subject_legacy_artifact_id": legacy_id,
            "upstream_artifact_ids": [],
            "downstream_artifact_ids": []
        })
    };
    let output_name = output_name(output);
    Ok(json!({
        "artifact_id": canonical_identity.canonical_artifact_id,
        "legacy_artifact_id": canonical_identity.legacy_artifact_id,
        "artifact_sha256": output.sha256,
        "node_id": output.node_id,
        "output_name": output_name,
        "node_fingerprint": output.node_fingerprint,
        "path": output.path,
        "size_bytes": size_bytes,
        "payload_missing": payload_missing,
        "promotable": output.promotable,
        "provenance": {
            "graph_fingerprint": manifest.graph_fingerprint,
            "run_id": manifest.run_id,
            "attempt": 0
        },
        "identity_explain": {
            "artifact_id": canonical_identity.canonical_artifact_id,
            "canonical_artifact_id": canonical_identity.canonical_artifact_id,
            "legacy_artifact_id": canonical_identity.legacy_artifact_id,
            "composed_from": {
                "run_id": canonical_identity.run_id,
                "node_id": canonical_identity.node_id,
                "output_name": output_name,
                "node_fingerprint": canonical_identity.node_fingerprint,
                "artifact_sha256": canonical_identity.artifact_sha256,
                "path": canonical_identity.output_path
            },
            "hash_algorithm": "sha256",
            "identity_scope": "artifact content + provenance",
            "collision_safe": true
        },
        "lineage": lineage
    }))
}

pub(crate) fn check_engine(bin: &str) -> Value {
    match std::process::Command::new(bin).arg("--version").output() {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            json!({"status":"ok","version":version})
        }
        _ => json!({"status":"missing"}),
    }
}

pub(crate) fn verify_run(run_dir: &Path, deep: bool, strict: bool) -> Result<Value, ExitCode> {
    let mut errors = Vec::new();
    let mut invariant_violations = Vec::new();

    let manifest_path = run_dir.join("manifest.json");
    let manifest_data = fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?;
    let manifest: Manifest = serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;
    let manifest_json: Value =
        serde_json::from_str(&manifest_data).map_err(|_| ExitCode::from(3))?;

    if manifest.started_unix_ms > manifest.finished_unix_ms {
        errors.push("manifest timestamps are not monotonic".to_string());
    }
    if let Some(dir_name) = run_dir.file_name().and_then(|value| value.to_str()) {
        if let Some(expected) = dir_name.strip_prefix("run-") {
            if manifest.run_id != expected {
                errors.push("manifest run_id does not match finalized run directory".to_string());
            }
        }
    }

    let schema_index_path = run_dir.join("run.schema.json");
    let schema_index = if schema_index_path.exists() {
        Some(read_typed_json::<RunDirSchemaIndex>(&schema_index_path)?)
    } else {
        None
    };
    let default_schema_index = RunDirSchemaIndex::default();
    let schema_index_ref = schema_index.as_ref().unwrap_or(&default_schema_index);

    let finalized_manifest_path = run_dir.join("manifest.finalized.json");
    let complete_marker_path = run_dir.join(".run-complete.json");
    let incomplete_marker_path = run_dir.join(".run-incomplete.json");
    if finalized_manifest_path.exists() {
        let finalized: Value = read_typed_json(&finalized_manifest_path)?;
        if finalized != manifest_json {
            errors.push("manifest.finalized.json does not match manifest.json".to_string());
        }
    }
    if complete_marker_path.exists() {
        let marker: Value = read_typed_json(&complete_marker_path)?;
        if marker.get("status").and_then(Value::as_str) != Some("complete") {
            errors.push(".run-complete.json does not mark the run as complete".to_string());
        }
        if marker.get("manifest").and_then(Value::as_str) != Some("manifest.finalized.json") {
            errors.push(".run-complete.json does not point to manifest.finalized.json".to_string());
        }
    }
    if incomplete_marker_path.exists() {
        errors.push(".run-incomplete.json is present; run must be repaired or resumed before it can be considered complete".to_string());
    }

    let required_root_files = schema_index_ref
        .required_root_files
        .iter()
        .map(|rel| (rel.clone(), run_dir.join(rel).exists()))
        .collect::<BTreeMap<_, _>>();
    let missing_root_files = required_root_files
        .iter()
        .filter_map(|(rel, present)| (!present).then_some(rel.clone()))
        .collect::<Vec<_>>();

    let snapshot = load_snapshot(run_dir)?;
    let computed = snapshot.graph.graph_fingerprint().unwrap_or_default();
    if computed != snapshot.graph_fingerprint {
        errors.push(format!(
            "graph_fingerprint mismatch: {} != {}",
            computed, snapshot.graph_fingerprint
        ));
    }
    for node in &snapshot.graph.nodes {
        if manifest.policy.deny_network && node.effects.contains(&Effect::Network) {
            errors.push(format!("policy deny_network violated by node {}", node.id));
        }
        if manifest.policy.deny_env && node.effects.contains(&Effect::Env) {
            errors.push(format!("policy deny_env violated by node {}", node.id));
        }
        if manifest.policy.deny_clock && node.effects.contains(&Effect::Clock) {
            errors.push(format!("policy deny_clock violated by node {}", node.id));
        }
    }

    let outputs_index_path = run_dir.join("outputs").join("index.json");
    let mut outputs_count = 0usize;
    let mut produced_artifacts = Vec::new();
    let mut produced_legacy_ids = BTreeSet::new();
    let produced_node_ids =
        snapshot.graph.nodes.iter().map(|node| node.id.clone()).collect::<BTreeSet<_>>();
    if !outputs_index_path.exists() {
        errors.push("missing outputs/index.json".to_string());
    } else {
        let index: RunOutputsIndex = read_typed_json(&outputs_index_path)?;
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
                errors.push(format!("output path is not normalized relative path: {}", file.path));
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
            let canonical = artifact_identity(&manifest.run_id, &file);
            let legacy = legacy_artifact_id(&file);
            produced_legacy_ids.insert(legacy.clone());
            produced_artifacts.push(json!({
                "artifact_id": canonical.canonical_artifact_id,
                "legacy_artifact_id": legacy,
                "node_id": file.node_id,
                "path": file.path,
            }));
        }
    }

    let nodes_dir = run_dir.join("nodes");
    let mut observed_statuses = Vec::new();
    let mut per_node = Vec::new();
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|entry| entry.ok())
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let trace_path = entry.path().join("trace.json");
            if !trace_path.exists() {
                errors.push(format!("missing trace: {node_id}"));
                continue;
            }
            let data = fs::read_to_string(&trace_path).map_err(|_| ExitCode::from(3))?;
            let val: Value = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
            if let Some(status) = val.get("status").and_then(|status| status.as_str()) {
                match status {
                    "success" => observed_statuses.push(NodeStatus::Success),
                    "failed" => observed_statuses.push(NodeStatus::Failed),
                    "skipped" => observed_statuses.push(NodeStatus::Skipped),
                    "cached" => observed_statuses.push(NodeStatus::Cached),
                    "cancelled" => observed_statuses.push(NodeStatus::Cancelled),
                    _ => {}
                }
            }
            if deep {
                let typed_parse: Result<bijux_dag_artifacts::NodeTrace, _> =
                    serde_json::from_str(&data);
                if typed_parse.is_err() {
                    errors.push(format!("trace schema parse failed: {node_id}"));
                }
            }
            for key in ["node_id", "status", "started_unix_ms", "finished_unix_ms", "fingerprint"] {
                if val.get(key).is_none() {
                    errors.push(format!("trace missing {key}: {node_id}"));
                }
            }
            if deep {
                let started = val.get("started_unix_ms").and_then(Value::as_u64).unwrap_or(0);
                let finished = val.get("finished_unix_ms").and_then(Value::as_u64).unwrap_or(0);
                if !trace_time_order_ok(started, finished) {
                    invariant_violations.push(format!("INV-TRACE-TIME-001 violation in {node_id}"));
                }
            }

            let required_files = schema_index_ref
                .required_node_files
                .iter()
                .map(|rel| (rel.clone(), entry.path().join(rel).exists()))
                .collect::<BTreeMap<_, _>>();
            let mut missing = required_files
                .iter()
                .filter_map(|(rel, present)| (!present).then_some(rel.clone()))
                .collect::<Vec<_>>();
            if val.get("status").and_then(Value::as_str) == Some("failed")
                && val.get("failure").is_none()
            {
                missing.push("failure_info".to_string());
            }
            if val.get("status").and_then(Value::as_str) == Some("cached")
                && val.get("cache_proof").is_none()
            {
                missing.push("cache_proof".to_string());
            }
            per_node.push(json!({
                "node_id": node_id,
                "status": val.get("status").cloned().unwrap_or(Value::Null),
                "required_files": required_files,
                "missing_evidence": missing,
            }));
        }
    }

    let manifest_counts = RunNodeCounts {
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
        cancelled: manifest.node_counts.cancelled,
    };
    if !run_summary_invariant_ok(manifest_counts, &observed_statuses) {
        invariant_violations
            .push("INV-RUN-COUNTS-001 manifest totals do not match node traces".to_string());
    }
    if manifest.status == "completed" && !terminal_run_has_terminal_node(&observed_statuses) {
        invariant_violations
            .push("INV-RUN-TERMINAL-001 completed run has no terminal node statuses".to_string());
    }

    if deep || strict {
        if serde_json::from_str::<Manifest>(&manifest_data).is_err() {
            errors.push("manifest schema parse failed".to_string());
        }
        if !outputs_index_path.exists() {
            errors.push("deep verify requires outputs/index.json".to_string());
        }
    }
    if strict {
        for rel in ["graph.snapshot.json", "nodes"] {
            if !run_dir.join(rel).exists() {
                errors.push(format!("strict verify missing required run artifact: {rel}"));
            }
        }
        if manifest.manifest_version != "run-manifest/v0.1" {
            errors.push("strict verify unsupported manifest_version".to_string());
        }
        for rel in ["observability.timeline.json", "observability.events.json"] {
            if !run_dir.join(rel).exists() {
                errors.push(format!("strict verify missing required run artifact: {rel}"));
            }
        }
        if manifest.status == "failed" && !run_dir.join("observability.root-causes.json").exists() {
            errors.push(
                "strict verify missing required run artifact: observability.root-causes.json"
                    .to_string(),
            );
        }
        for rel in &missing_root_files {
            errors.push(format!("strict verify missing required run artifact: {rel}"));
        }
    }
    if deep || strict {
        if let Some(summary) = manifest_json
            .get("run_metadata")
            .and_then(|metadata| metadata.get("environment_summary"))
        {
            let summary_bytes = serde_json::to_vec(summary).map_err(|_| ExitCode::from(3))?;
            let expected = sha256_bytes(&summary_bytes);
            let actual = manifest_json
                .get("run_metadata")
                .and_then(|metadata| metadata.get("environment_summary_sha256"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if actual != expected {
                errors.push("environment summary checksum mismatch".to_string());
            }
        }
    }

    let lineage_completeness = if run_dir.join("lineage.snapshot.json").exists() {
        let snapshot: bijux_dag_artifacts::lineage::ArtifactLineageSnapshot =
            read_typed_json(&run_dir.join("lineage.snapshot.json"))?;
        let edge_ids =
            snapshot.edges.iter().map(|edge| edge.artifact_id.clone()).collect::<BTreeSet<_>>();
        let mut missing_producer_edges = produced_legacy_ids
            .iter()
            .filter(|artifact_id| !edge_ids.contains(*artifact_id))
            .cloned()
            .collect::<Vec<_>>();
        missing_producer_edges.sort();
        let mut unknown_upstream_artifact_ids = snapshot
            .edges
            .iter()
            .flat_map(|edge| edge.upstream_artifact_ids.iter())
            .filter(|artifact_id| {
                !produced_legacy_ids.contains(*artifact_id)
                    && !is_external_artifact_id(artifact_id)
                    && !is_known_wildcard_upstream(artifact_id, &produced_node_ids)
            })
            .cloned()
            .collect::<Vec<_>>();
        unknown_upstream_artifact_ids.sort();
        unknown_upstream_artifact_ids.dedup();
        json!({
            "edge_count": snapshot.edges.len(),
            "missing_producer_edges": missing_producer_edges,
            "unknown_upstream_artifact_ids": unknown_upstream_artifact_ids,
            "complete": !snapshot.edges.is_empty()
                && missing_producer_edges.is_empty()
                && unknown_upstream_artifact_ids.is_empty()
        })
    } else {
        json!({
            "edge_count": 0,
            "missing_producer_edges": produced_legacy_ids.iter().cloned().collect::<Vec<_>>(),
            "unknown_upstream_artifact_ids": [],
            "complete": false
        })
    };

    let event_log_completeness = if run_dir.join("observability.events.json").exists() {
        let events: Vec<EventRecord> = read_typed_json(&run_dir.join("observability.events.json"))?;
        let timeline = if run_dir.join("observability.timeline.json").exists() {
            Some(read_typed_json::<TimelineExport>(&run_dir.join("observability.timeline.json"))?)
        } else {
            None
        };
        let mut report =
            serde_json::to_value(verify_event_log_completeness(&events, timeline.as_ref()))
                .map_err(|_| ExitCode::from(3))?;
        let mut per_node_gaps = Vec::new();
        for node in &per_node {
            let Some(node_id) = node.get("node_id").and_then(Value::as_str) else {
                continue;
            };
            let node_status = node.get("status").and_then(Value::as_str).unwrap_or("unknown");
            let started = events
                .iter()
                .filter(|event| {
                    event.node_id.as_deref() == Some(node_id) && event.name == "node_started"
                })
                .count();
            let terminal_event_name = timeline_terminal_event_name(node_status);
            let finished = events
                .iter()
                .filter(|event| {
                    event.node_id.as_deref() == Some(node_id) && event.name == terminal_event_name
                })
                .count();
            if !matches!(node_status, "cached" | "skipped" | "cancelled") && started == 0 {
                per_node_gaps.push(format!("{node_id}: missing node_started event"));
            }
            if finished == 0 {
                per_node_gaps.push(format!("{node_id}: missing {terminal_event_name} event"));
            }
            if finished > 1 {
                per_node_gaps.push(format!("{node_id}: multiple {terminal_event_name} events"));
            }
        }
        if let Some(object) = report.as_object_mut() {
            object.insert(
                "reconstructed_timeline".to_string(),
                serde_json::to_value(reconstruct_timeline_from_events(&events))
                    .map_err(|_| ExitCode::from(3))?,
            );
            object.insert("per_node_gaps".to_string(), json!(per_node_gaps));
        }
        report
    } else {
        json!({
            "complete": false,
            "required_names_present": false,
            "required_timeline_labels_present": false,
            "required_event_field_gaps": [],
            "missing_required_names": [],
            "missing_required_timeline_labels": [],
            "monotonic_timestamps": false,
            "timeline_matches_reconstruction": false,
            "gaps": ["missing observability.events.json"],
            "per_node_gaps": ["missing observability.events.json"]
        })
    };

    let status = if errors.is_empty() && invariant_violations.is_empty() { "ok" } else { "error" };
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
            "outputs_files": outputs_count,
            "schema_index": schema_index_path.exists(),
            "manifest_finalized": finalized_manifest_path.exists(),
            "run_complete_marker": complete_marker_path.exists()
                && !incomplete_marker_path.exists()
        },
        "evidence_completeness": {
            "required_root_files": required_root_files,
            "missing_root_files": missing_root_files,
            "per_node": per_node,
        },
        "produced_artifacts": produced_artifacts,
        "lineage_completeness": lineage_completeness,
        "event_log_completeness": event_log_completeness,
        "errors": errors,
        "invariant_violations": invariant_violations
    }))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}
