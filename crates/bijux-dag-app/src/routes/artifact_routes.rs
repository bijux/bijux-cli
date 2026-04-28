use crate::commands::{ArtifactCommands, DagCli};
use crate::routes::run_lookup::read_manifest_json;
use crate::{emit_json, inspect_artifact, read_file, ExitCode};
use bijux_dag_artifacts::platform::{
    compact_lineage, lineage_dependencies, lineage_dependents,
};
use bijux_dag_artifacts::retention::RetentionPolicy;
use bijux_dag_artifacts::lineage::ArtifactLineageSnapshot;
use bijux_dag_artifacts::{ArtifactCleanupPlan, RunOutputsIndex};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
struct RegistryArtifactEntry {
    artifact_id: String,
    node_id: String,
    node_fingerprint: String,
    path: String,
    sha256: String,
    size_bytes: Option<u64>,
    payload_missing: bool,
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
                artifact_id: format!(
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

fn artifact_payload_path(run_dir: &Path, artifact_id: &str) -> Result<std::path::PathBuf, ExitCode> {
    let (node_id, file_name) = artifact_id.split_once(':').ok_or(ExitCode::from(2))?;
    let index_raw = read_file(&run_dir.join("outputs").join("index.json"))?;
    let index = serde_json::from_str::<serde_json::Value>(&index_raw).map_err(|_| ExitCode::from(3))?;
    let relative = index
        .get("files")
        .and_then(|value| value.as_array())
        .and_then(|files| {
            files.iter().find_map(|file| {
                let file_node_id = file.get("node_id").and_then(|value| value.as_str())?;
                let path = file.get("path").and_then(|value| value.as_str())?;
                if file_node_id == node_id && path.ends_with(&format!("/{file_name}")) {
                    Some(path.to_string())
                } else {
                    None
                }
            })
        })
        .ok_or(ExitCode::from(3))?;
    Ok(run_dir.join(relative))
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
                    if report.missing_payloads == 0 { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if report.missing_payloads == 0 { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
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
    use super::{artifact_registry_report, handle_artifact_command, handle_artifact_inspect_command};
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
            r#"{"run_id":"run-01","status":"success","graph_fingerprint":"fp"}"#,
        )
        .expect("manifest");
        std::fs::write(
            run.join("outputs").join("index.json"),
            r#"{"files":[{"node_id":"extract","node_fingerprint":"fp-node","sha256":"abc","path":"nodes/extract/outputs/report.json"}]}"#,
        )
        .expect("outputs index");
        std::fs::create_dir_all(run.join("nodes").join("extract").join("outputs")).expect("node outputs");
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
    fn artifact_inspect_route_rejects_missing_run_without_panic() {
        let cli = DagCli::parse_from(["dag", "artifact-inspect", "/missing/run", "n1:out"]);
        let result = handle_artifact_inspect_command(&cli, Path::new("/missing/run"), "n1:out");
        assert!(result.is_err());
    }
}
