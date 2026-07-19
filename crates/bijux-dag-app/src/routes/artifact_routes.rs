use crate::commands::{ArtifactCommands, DagCli};
use crate::routes::run_lookup::read_manifest_json;
use crate::{emit_json, inspect_artifact, read_file, ExitCode};
use bijux_dag_artifacts::lineage::ArtifactLineageSnapshot;
use bijux_dag_artifacts::platform::{compact_lineage, lineage_dependencies, lineage_dependents};
use bijux_dag_artifacts::retention::RetentionPolicy;
use bijux_dag_artifacts::{
    append_promotion_record, append_promotion_summary, build_artifact_identity,
    build_promoted_output_summary, now_unix_ms, promotion_record_path, sha256_artifact_path,
    write_json_atomic_durable, ArtifactCleanupPlan, ArtifactPromotionRecord, Manifest,
    PromotionEnvironment, PromotionLineageSummary, RunOutputFile, RunOutputsIndex,
};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
struct RegistryArtifactEntry {
    artifact_id: String,
    legacy_artifact_id: String,
    node_id: String,
    node_fingerprint: String,
    path: String,
    sha256: String,
    size_bytes: Option<u64>,
    payload_missing: bool,
    promotable: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ArtifactRegistryReport {
    run_id: String,
    total_artifacts: usize,
    nodes_with_artifacts: usize,
    missing_payloads: usize,
    artifacts: Vec<RegistryArtifactEntry>,
}

fn read_outputs_index(run_dir: &Path) -> Result<RunOutputsIndex, ExitCode> {
    let raw = read_file(&run_dir.join("outputs").join("index.json"))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn read_typed_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn read_lineage_snapshot(run_dir: &Path) -> Result<ArtifactLineageSnapshot, ExitCode> {
    let raw = read_file(&run_dir.join("lineage.snapshot.json"))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn enumerate_top_level_entries(root: &Path) -> Result<Vec<String>, ExitCode> {
    let mut entries: Vec<String> = fs::read_dir(root)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();
    Ok(entries)
}

fn build_retention_plan(root: &Path) -> Result<ArtifactCleanupPlan, ExitCode> {
    let policy = RetentionPolicy::default();
    let entries = enumerate_top_level_entries(root)?;
    Ok(bijux_dag_artifacts::build_cleanup_plan(&entries, &policy.retain_prefixes()))
}

fn artifact_registry_report(run_dir: &Path) -> Result<ArtifactRegistryReport, ExitCode> {
    let manifest = read_manifest_json(run_dir)?;
    let run_id = manifest.get("run_id").and_then(|v| v.as_str()).unwrap_or("unknown-run");
    let index = read_outputs_index(run_dir)?;
    let mut artifacts = index
        .files
        .iter()
        .map(|file| {
            let payload_path = run_dir.join(&file.path);
            let metadata = fs::metadata(&payload_path);
            RegistryArtifactEntry {
                artifact_id: build_artifact_identity(
                    run_id,
                    &file.node_id,
                    &file.path,
                    &file.node_fingerprint,
                    &file.sha256,
                )
                .canonical_artifact_id,
                legacy_artifact_id: format!(
                    "{}:{}",
                    file.node_id,
                    Path::new(&file.path)
                        .file_name()
                        .and_then(|v| v.to_str())
                        .unwrap_or(file.path.as_str())
                ),
                node_id: file.node_id.clone(),
                node_fingerprint: file.node_fingerprint.clone(),
                path: file.path.clone(),
                sha256: file.sha256.clone(),
                size_bytes: metadata.as_ref().ok().map(|m| m.len()),
                payload_missing: metadata.is_err(),
                promotable: file.promotable,
            }
        })
        .collect::<Vec<_>>();
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    let missing_payloads = artifacts.iter().filter(|entry| entry.payload_missing).count();
    let nodes_with_artifacts =
        artifacts.iter().map(|entry| entry.node_id.clone()).collect::<BTreeSet<_>>().len();
    Ok(ArtifactRegistryReport {
        run_id: run_id.to_string(),
        total_artifacts: artifacts.len(),
        nodes_with_artifacts,
        missing_payloads,
        artifacts,
    })
}

fn artifact_payload_path(
    run_dir: &Path,
    artifact_id: &str,
) -> Result<std::path::PathBuf, ExitCode> {
    Ok(run_dir.join(resolve_artifact_output(run_dir, artifact_id)?.path))
}

fn resolve_artifact_output(run_dir: &Path, artifact_id: &str) -> Result<RunOutputFile, ExitCode> {
    let manifest: Manifest = read_typed_json(&run_dir.join("manifest.json"))?;
    let index = read_outputs_index(run_dir)?;
    index
        .files
        .into_iter()
        .find(|file| {
            let canonical = build_artifact_identity(
                &manifest.run_id,
                &file.node_id,
                &file.path,
                &file.node_fingerprint,
                &file.sha256,
            );
            let legacy = format!(
                "{}:{}",
                file.node_id,
                Path::new(&file.path)
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or(file.path.as_str())
            );
            artifact_id == legacy || artifact_id == canonical.canonical_artifact_id
        })
        .ok_or(ExitCode::from(3))
}

fn parse_promotion_environment(label: &str) -> PromotionEnvironment {
    match label.trim().to_ascii_lowercase().as_str() {
        "local" => PromotionEnvironment::Local,
        "staging" => PromotionEnvironment::Staging,
        "release" => PromotionEnvironment::Release,
        other => PromotionEnvironment::Custom(other.to_string()),
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;
        if metadata.file_type().is_symlink() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("promotion source must not include symlinks: {}", source_path.display()),
            ));
        }
        if metadata.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
        }
    }
    Ok(())
}

fn deliverable_dir(
    deliverables_root: &Path,
    target_environment: &PromotionEnvironment,
    run_id: &str,
    node_id: &str,
    output_name: &str,
) -> std::path::PathBuf {
    deliverables_root.join(target_environment.label()).join(run_id).join(node_id).join(output_name)
}

fn write_manifest_with_promotion_summary(
    run_dir: &Path,
    summary: bijux_dag_artifacts::PromotedOutputSummary,
) -> Result<(), ExitCode> {
    let mut manifest: Manifest = read_typed_json(&run_dir.join("manifest.json"))?;
    append_promotion_summary(&mut manifest, summary);
    let manifest_value = serde_json::to_value(&manifest).map_err(|_| ExitCode::from(3))?;
    write_json_atomic_durable(run_dir.join("manifest.json"), &manifest_value)
        .map_err(|_| ExitCode::from(3))?;
    let finalized_path = run_dir.join("manifest.finalized.json");
    if finalized_path.exists() {
        write_json_atomic_durable(finalized_path, &manifest_value)
            .map_err(|_| ExitCode::from(3))?;
    }
    Ok(())
}

fn promote_artifact(
    run_dir: &Path,
    artifact_id: &str,
    deliverables_root: &Path,
    to: &str,
) -> Result<serde_json::Value, ExitCode> {
    let manifest: Manifest = read_typed_json(&run_dir.join("manifest.json"))?;
    let output = resolve_artifact_output(run_dir, artifact_id)?;
    if !output.promotable {
        return Err(ExitCode::from(2));
    }
    let details = inspect_artifact(run_dir, artifact_id)?;
    let canonical_artifact_id = details
        .get("artifact_id")
        .and_then(|value| value.as_str())
        .ok_or(ExitCode::from(3))?
        .to_string();
    let legacy_artifact_id = details
        .get("legacy_artifact_id")
        .and_then(|value| value.as_str())
        .ok_or(ExitCode::from(3))?
        .to_string();

    let source_payload = run_dir.join(&output.path);
    if !source_payload.exists() {
        return Err(ExitCode::from(3));
    }
    let verified_sha256 = sha256_artifact_path(&source_payload).map_err(|_| ExitCode::from(3))?;
    if verified_sha256 != output.sha256 {
        return Err(ExitCode::from(3));
    }

    let target_environment = parse_promotion_environment(to);
    let destination_dir = deliverable_dir(
        deliverables_root,
        &target_environment,
        &manifest.run_id,
        &output.node_id,
        &output.name,
    );
    let record_path = destination_dir.join("promotion.json");
    let payload_file_name = Path::new(&output.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(output.name.as_str());
    let payload_relpath = if source_payload.is_dir() {
        "payload".to_string()
    } else {
        format!("payload/{payload_file_name}")
    };
    let promoted_unix_ms = now_unix_ms();
    let record = ArtifactPromotionRecord {
        schema_version: "artifact-promotion/v0.1".to_string(),
        canonical_artifact_id: canonical_artifact_id.clone(),
        legacy_artifact_id: legacy_artifact_id.clone(),
        source_run_id: manifest.run_id.clone(),
        source_node_id: output.node_id.clone(),
        source_output_name: output.name.clone(),
        source_output_path: output.path.clone(),
        artifact_sha256: output.sha256.clone(),
        payload_kind: output.kind.clone(),
        payload_relpath: payload_relpath.clone(),
        destination_path: destination_dir.display().to_string(),
        from: PromotionEnvironment::Local,
        to: target_environment.clone(),
        promoted_unix_ms,
        lineage: PromotionLineageSummary {
            subject_artifact_id: details["lineage"]["subject_artifact_id"]
                .as_str()
                .unwrap_or(canonical_artifact_id.as_str())
                .to_string(),
            subject_legacy_artifact_id: details["lineage"]["subject_legacy_artifact_id"]
                .as_str()
                .unwrap_or(legacy_artifact_id.as_str())
                .to_string(),
            upstream_artifact_ids: details["lineage"]["upstream_artifact_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect(),
            downstream_artifact_ids: details["lineage"]["downstream_artifact_ids"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect(),
        },
    };

    if destination_dir.exists() {
        let existing: ArtifactPromotionRecord = read_typed_json(&record_path)?;
        if existing.canonical_artifact_id != record.canonical_artifact_id
            || existing.artifact_sha256 != record.artifact_sha256
        {
            return Err(ExitCode::from(3));
        }
    } else {
        let parent = destination_dir.parent().ok_or(ExitCode::from(3))?;
        fs::create_dir_all(parent).map_err(|_| ExitCode::from(3))?;
        let stage_dir = parent.join(format!(".promotion-{}-{}", output.name, promoted_unix_ms));
        if stage_dir.exists() {
            fs::remove_dir_all(&stage_dir).map_err(|_| ExitCode::from(3))?;
        }
        fs::create_dir_all(&stage_dir).map_err(|_| ExitCode::from(3))?;
        let staged_payload_root = stage_dir.join("payload");
        if source_payload.is_dir() {
            copy_dir_recursive(&source_payload, &staged_payload_root)
                .map_err(|_| ExitCode::from(3))?;
        } else {
            fs::create_dir_all(&staged_payload_root).map_err(|_| ExitCode::from(3))?;
            fs::copy(&source_payload, staged_payload_root.join(payload_file_name))
                .map_err(|_| ExitCode::from(3))?;
        }
        let record_value = serde_json::to_value(&record).map_err(|_| ExitCode::from(3))?;
        write_json_atomic_durable(stage_dir.join("promotion.json"), &record_value)
            .map_err(|_| ExitCode::from(3))?;
        fs::rename(&stage_dir, &destination_dir).map_err(|_| ExitCode::from(3))?;
    }

    append_promotion_record(run_dir, &record).map_err(|_| ExitCode::from(3))?;
    let summary = build_promoted_output_summary(&record);
    write_manifest_with_promotion_summary(run_dir, summary.clone())?;

    Ok(json!({
        "artifact_id": record.canonical_artifact_id,
        "legacy_artifact_id": record.legacy_artifact_id,
        "artifact_sha256": record.artifact_sha256,
        "run_id": record.source_run_id,
        "node_id": record.source_node_id,
        "output_name": record.source_output_name,
        "destination": destination_dir,
        "payload_relpath": record.payload_relpath,
        "target_environment": target_environment.label(),
        "record_path": record_path,
        "run_record_path": promotion_record_path(run_dir, &canonical_artifact_id),
        "lineage": record.lineage,
        "promotable": output.promotable
    }))
}

pub(crate) fn handle_artifact_inspect_command(
    cli: &DagCli,
    run_dir: &Path,
    artifact_id: &str,
) -> Result<ExitCode, ExitCode> {
    let details = inspect_artifact(run_dir, artifact_id)?;
    if cli.json {
        return emit_json(
            cli,
            "dag.artifact-inspect",
            true,
            details,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    crate::routes::renderer::print_pretty_json(&details);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_artifact_command(
    cli: &DagCli,
    command: &ArtifactCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ArtifactCommands::Fetch { run_dir, artifact_id, out } => {
            let source = artifact_payload_path(run_dir, artifact_id)?;
            let parent = out.parent().unwrap_or_else(|| Path::new("."));
            fs::create_dir_all(parent).map_err(|_| ExitCode::from(3))?;
            fs::copy(&source, out).map_err(|_| ExitCode::from(3))?;
            let payload = json!({
                "artifact_id": artifact_id,
                "source": source,
                "out": out,
            });
            if cli.json {
                return emit_json(
                    cli,
                    "dag.artifact.fetch",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ArtifactCommands::Registry { run_dir } => {
            let report = artifact_registry_report(run_dir)?;
            let payload = serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.artifact.registry",
                    report.missing_payloads == 0,
                    payload,
                    if report.missing_payloads == 0 {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"artifact_registry_missing_payloads",
                            "severity":"error",
                            "message":"artifact registry references missing payload files",
                        })]
                    },
                    if report.missing_payloads == 0 {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if report.missing_payloads == 0 {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        ArtifactCommands::Lineage { run_dir, artifact_id } => {
            let snapshot = read_lineage_snapshot(run_dir)?;
            let compacted = compact_lineage(&snapshot);
            let payload = if let Some(artifact_id) = artifact_id {
                json!({
                    "schema_version": snapshot.schema_version,
                    "artifact_count": compacted.artifact_count,
                    "edge_count": compacted.edge_count,
                    "producer_index": compacted.producer_index,
                    "artifact_id": artifact_id,
                    "upstream_artifact_ids": lineage_dependencies(&snapshot, artifact_id),
                    "downstream_artifact_ids": lineage_dependents(&snapshot, artifact_id),
                })
            } else {
                serde_json::to_value(&compacted).map_err(|_| ExitCode::from(3))?
            };
            if cli.json {
                return emit_json(
                    cli,
                    "dag.artifact.lineage",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ArtifactCommands::Promote { run_dir, artifact_id, deliverables_root, to } => {
            let payload = promote_artifact(run_dir, artifact_id, deliverables_root, to)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.artifact.promote",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ArtifactCommands::Retention { root } => {
            let policy = RetentionPolicy::default();
            let entries = enumerate_top_level_entries(root)?;
            let plan = build_retention_plan(root)?;
            let payload = json!({
                "policy": policy,
                "entry_count": entries.len(),
                "entries": entries,
                "retained": plan.retained,
                "prunable": plan.prunable,
            });
            if cli.json {
                return emit_json(
                    cli,
                    "dag.artifact.retention",
                    true,
                    payload,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        artifact_registry_report, handle_artifact_command, handle_artifact_inspect_command,
    };
    use crate::commands::{ArtifactCommands, Commands, DagCli};
    use crate::ExitCode;
    use clap::Parser;
    use std::path::Path;

    fn quiet_json_cli(command: ArtifactCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Artifact { command } }
    }

    fn write_run_fixture(run: &std::path::Path) {
        std::fs::create_dir_all(run.join("outputs")).expect("outputs");
        std::fs::write(
            run.join("manifest.json"),
            r#"{
              "manifest_version":"run-manifest/v0.1",
              "run_id":"run-01",
              "created_unix_ms":1,
              "started_unix_ms":1,
              "finished_unix_ms":2,
              "graph_snapshot":"graph.snapshot.json",
              "status":"success",
              "spec":"bijux-dag/v0.1",
              "graph_fingerprint":"fp",
              "planner_contract_version":"planner-contract/v0.1",
              "tool_version":"0.4.0",
              "jobs":1,
              "adapters":[],
              "outputs":[],
              "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0,"cancelled":0},
              "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
            }"#,
        )
        .expect("manifest");
        std::fs::write(
            run.join("outputs").join("index.json"),
            r#"{"files":[{"node_id":"extract","node_fingerprint":"fp-node","name":"report","kind":"file","media_type":"application/json","size_bytes":2,"sha256":"44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a","path":"nodes/extract/outputs/report.json","promotable":true}]}"#,
        )
        .expect("outputs index");
        std::fs::create_dir_all(run.join("nodes").join("extract").join("outputs"))
            .expect("node outputs");
        std::fs::write(
            run.join("nodes").join("extract").join("outputs").join("report.json"),
            b"{}",
        )
        .expect("payload");
        std::fs::write(
            run.join("lineage.snapshot.json"),
            r#"{
              "schema_version":"v0.1",
              "edges":[
                {
                  "artifact_id":"extract:report.json",
                  "producer_node_id":"extract",
                  "upstream_artifact_ids":["seed:input.csv"]
                }
              ]
            }"#,
        )
        .expect("lineage");
    }

    #[test]
    fn artifact_registry_report_flags_missing_payloads() {
        let dir = tempfile::tempdir().expect("tmp");
        write_run_fixture(dir.path());
        std::fs::remove_file(
            dir.path().join("nodes").join("extract").join("outputs").join("report.json"),
        )
        .expect("remove payload");
        let report = artifact_registry_report(dir.path()).expect("registry");
        assert_eq!(report.total_artifacts, 1);
        assert_eq!(report.missing_payloads, 1);
    }

    #[test]
    fn artifact_routes_support_registry_lineage_retention_and_fetch() {
        let dir = tempfile::tempdir().expect("tmp");
        write_run_fixture(dir.path());
        std::fs::create_dir_all(dir.path().join("run-2026-01-01")).expect("run old");
        std::fs::write(dir.path().join("tmp-upload"), b"garbage").expect("tmp");

        let registry_cli =
            quiet_json_cli(ArtifactCommands::Registry { run_dir: dir.path().to_path_buf() });
        let registry = handle_artifact_command(
            &registry_cli,
            &ArtifactCommands::Registry { run_dir: dir.path().to_path_buf() },
        )
        .expect("registry");
        assert_eq!(registry, ExitCode::SUCCESS);

        let lineage_cli = quiet_json_cli(ArtifactCommands::Lineage {
            run_dir: dir.path().to_path_buf(),
            artifact_id: Some("extract:report.json".to_string()),
        });
        let lineage = handle_artifact_command(
            &lineage_cli,
            &ArtifactCommands::Lineage {
                run_dir: dir.path().to_path_buf(),
                artifact_id: Some("extract:report.json".to_string()),
            },
        )
        .expect("lineage");
        assert_eq!(lineage, ExitCode::SUCCESS);

        let retention_cli =
            quiet_json_cli(ArtifactCommands::Retention { root: dir.path().to_path_buf() });
        let retention = handle_artifact_command(
            &retention_cli,
            &ArtifactCommands::Retention { root: dir.path().to_path_buf() },
        )
        .expect("retention");
        assert_eq!(retention, ExitCode::SUCCESS);

        let fetch_out = dir.path().join("copied").join("report.json");
        let fetch_cli = quiet_json_cli(ArtifactCommands::Fetch {
            run_dir: dir.path().to_path_buf(),
            artifact_id: "extract:report.json".to_string(),
            out: fetch_out.clone(),
        });
        let fetch = handle_artifact_command(
            &fetch_cli,
            &ArtifactCommands::Fetch {
                run_dir: dir.path().to_path_buf(),
                artifact_id: "extract:report.json".to_string(),
                out: fetch_out.clone(),
            },
        )
        .expect("fetch");
        assert_eq!(fetch, ExitCode::SUCCESS);
        assert!(fetch_out.exists());
    }

    #[test]
    fn artifact_promote_copies_payload_and_updates_run_summary() {
        let dir = tempfile::tempdir().expect("tmp");
        write_run_fixture(dir.path());
        let deliverables = dir.path().join("deliverables");
        let cli = quiet_json_cli(ArtifactCommands::Promote {
            run_dir: dir.path().to_path_buf(),
            artifact_id: "extract:report.json".to_string(),
            deliverables_root: deliverables.clone(),
            to: "release".to_string(),
        });

        let exit = handle_artifact_command(
            &cli,
            &ArtifactCommands::Promote {
                run_dir: dir.path().to_path_buf(),
                artifact_id: "extract:report.json".to_string(),
                deliverables_root: deliverables.clone(),
                to: "release".to_string(),
            },
        )
        .expect("promote");
        assert_eq!(exit, ExitCode::SUCCESS);

        let promotion_dir =
            deliverables.join("release").join("run-01").join("extract").join("report");
        assert!(promotion_dir.join("payload").join("report.json").exists());
        let promotion: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(promotion_dir.join("promotion.json")).expect("promotion"),
        )
        .expect("promotion json");
        assert_eq!(
            promotion["artifact_sha256"].as_str(),
            Some("44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a")
        );
        assert_eq!(
            promotion["lineage"]["upstream_artifact_ids"][0].as_str(),
            Some("seed:input.csv")
        );

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join("manifest.json")).expect("manifest"),
        )
        .expect("manifest json");
        assert_eq!(
            manifest["run_summary"]["promoted_outputs"][0]["output_name"].as_str(),
            Some("report")
        );
        assert!(dir.path().join("promotions").join("index.json").exists());
    }

    #[test]
    fn artifact_promote_rejects_corrupt_source_payload() {
        let dir = tempfile::tempdir().expect("tmp");
        write_run_fixture(dir.path());
        std::fs::write(
            dir.path().join("nodes").join("extract").join("outputs").join("report.json"),
            b"{\"corrupt\":true}",
        )
        .expect("corrupt payload");

        let deliverables = dir.path().join("deliverables");
        let cli = quiet_json_cli(ArtifactCommands::Promote {
            run_dir: dir.path().to_path_buf(),
            artifact_id: "extract:report.json".to_string(),
            deliverables_root: deliverables.clone(),
            to: "release".to_string(),
        });
        let err = handle_artifact_command(
            &cli,
            &ArtifactCommands::Promote {
                run_dir: dir.path().to_path_buf(),
                artifact_id: "extract:report.json".to_string(),
                deliverables_root: deliverables,
                to: "release".to_string(),
            },
        )
        .expect_err("corruption must fail");
        assert_eq!(err, ExitCode::from(3));
    }

    #[test]
    fn artifact_promote_rejects_outputs_that_are_not_marked_promotable() {
        let dir = tempfile::tempdir().expect("tmp");
        write_run_fixture(dir.path());
        let index_path = dir.path().join("outputs").join("index.json");
        std::fs::write(
            &index_path,
            r#"{"files":[{"node_id":"extract","node_fingerprint":"fp-node","name":"report","kind":"file","media_type":"application/json","size_bytes":2,"sha256":"44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a","path":"nodes/extract/outputs/report.json","promotable":false}]}"#,
        )
        .expect("index");

        let deliverables = dir.path().join("deliverables");
        let cli = quiet_json_cli(ArtifactCommands::Promote {
            run_dir: dir.path().to_path_buf(),
            artifact_id: "extract:report.json".to_string(),
            deliverables_root: deliverables,
            to: "release".to_string(),
        });
        let err = handle_artifact_command(
            &cli,
            &ArtifactCommands::Promote {
                run_dir: dir.path().to_path_buf(),
                artifact_id: "extract:report.json".to_string(),
                deliverables_root: dir.path().join("deliverables"),
                to: "release".to_string(),
            },
        )
        .expect_err("non-promotable outputs must be rejected");
        assert_eq!(err, ExitCode::from(2));
    }

    #[test]
    fn artifact_inspect_route_rejects_missing_run_without_panic() {
        let cli = DagCli::parse_from(["bijux-dag", "artifact-inspect", "/missing/run", "n1:out"]);
        let result = handle_artifact_inspect_command(&cli, Path::new("/missing/run"), "n1:out");
        assert!(result.is_err());
    }
}
