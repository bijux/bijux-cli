use crate::commands::DagCli;
use crate::{
    collect_output_files, emit_json, read_file, read_node_traces, read_outputs_indexes,
    verify_bundle_invariants, ExitCode, Value,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_export_command(
    cli: &DagCli,
    run_dir: &Option<std::path::PathBuf>,
    from_run: &Option<std::path::PathBuf>,
    out: &Path,
    manifest_only: bool,
    without_artifacts: bool,
    provenance_only: bool,
    redact: bool,
    with_files: bool,
    include_files: bool,
) -> Result<ExitCode, ExitCode> {
    let resolved_run_dir = match (run_dir, from_run) {
        (Some(positional), None) => positional.clone(),
        (None, Some(flagged)) => flagged.clone(),
        (Some(positional), Some(flagged)) => {
            if positional == flagged {
                positional.clone()
            } else {
                return Err(ExitCode::from(2));
            }
        }
        (None, None) => return Err(ExitCode::from(2)),
    };
    let include_files_effective = with_files || include_files;
    if manifest_only && include_files_effective {
        return Err(ExitCode::from(2));
    }
    if without_artifacts && include_files_effective {
        return Err(ExitCode::from(2));
    }
    if provenance_only && include_files_effective {
        return Err(ExitCode::from(2));
    }
    let manifest = read_file(&resolved_run_dir.join("manifest.json"))?;
    let snapshot = read_file(&resolved_run_dir.join("graph.snapshot.json"))?;
    let nodes = if provenance_only {
        HashMap::new()
    } else {
        read_node_traces(&resolved_run_dir)?
    };
    let outputs = if without_artifacts || provenance_only {
        Default::default()
    } else {
        read_outputs_indexes(&resolved_run_dir)?
    };
    let files = if include_files_effective && !without_artifacts && !provenance_only {
        Some(collect_output_files(&resolved_run_dir, &outputs)?)
    } else {
        None
    };
    let export_mode = if provenance_only {
        "provenance-only"
    } else if without_artifacts {
        "without-artifacts"
    } else if include_files_effective {
        "with-files"
    } else {
        "manifest-only"
    };
    let source_run_dir = if redact {
        Value::String("[redacted]".to_string())
    } else {
        json!(resolved_run_dir)
    };
    let bundle = json!({
        "bundle_version": "export-bundle/v0.1",
        "export_mode": export_mode,
        "provenance": {
            "source": "native-run",
            "imported": false,
            "source_run_dir": source_run_dir,
        },
        "manifest": serde_json::from_str::<serde_json::Value>(&manifest).ok(),
        "graph_snapshot": serde_json::from_str::<serde_json::Value>(&snapshot).ok(),
        "node_traces": nodes,
        "outputs": outputs,
        "files": files,
    });
    let bundle_invariant_violations = verify_bundle_invariants(&bundle);
    if !bundle_invariant_violations.is_empty() {
        return Err(ExitCode::from(3));
    }
    fs::write(out, serde_json::to_vec_pretty(&bundle).unwrap()).map_err(|_| ExitCode::from(3))?;
    if cli.json {
        return emit_json(
            cli,
            "dag.export",
            true,
            json!({ "bundle": out }),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    } else if !cli.quiet {
        println!("bundle: {}", out.display());
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_import_command(
    cli: &DagCli,
    file: &Path,
    verify_only: bool,
) -> Result<ExitCode, ExitCode> {
    let data = read_file(file)?;
    let val: serde_json::Value = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
    let bundle_version = val
        .get("bundle_version")
        .or_else(|| val.get("export_bundle_version"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if bundle_version != "export-bundle/v0.1" {
        let summary = json!({
            "error": "unsupported export bundle version",
            "supported": "export-bundle/v0.1",
            "found": bundle_version
        });
        if cli.json {
            return emit_json(
                cli,
                "dag.import",
                false,
                summary,
                vec![
                    json!({"message":"unsupported bundle version","remediation":"export with export-bundle/v0.1"}),
                ],
                ExitCode::from(3),
            );
        }
        println!("import summary: {}", summary);
        return Err(ExitCode::from(3));
    }
    let mut invariant_violations = verify_bundle_invariants(&val);
    if val.get("bundle_version").is_none() && val.get("export_bundle_version").is_some() {
        invariant_violations.retain(|v| {
            !v.starts_with("INV-EXPORT-VERSION-001")
                && !v.starts_with("INV-EXPORT-MODE-001")
                && !v.starts_with("INV-EXPORT-VERIFY-001 missing graph_snapshot")
                && !v.starts_with("INV-EXPORT-VERIFY-001 missing outputs map")
        });
    }
    let nodes = val
        .get("node_traces")
        .and_then(|v| v.as_object())
        .map(|o| o.len())
        .unwrap_or(0);
    let failed = val
        .get("node_traces")
        .and_then(|v| v.as_object())
        .map(|o| {
            o.iter()
                .filter_map(|(k, v)| {
                    if v.get("status") == Some(&serde_json::Value::String("failed".to_string())) {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let preservation_lineage = val
        .get("provenance")
        .and_then(|v| v.get("lineage"))
        .is_some();
    let preservation_run_ancestry = val
        .get("provenance")
        .and_then(|v| v.get("parent_run_id"))
        .is_some()
        || val
            .get("provenance")
            .and_then(|v| v.get("source_run_id"))
            .is_some();
    let preservation_graph_identity = val.get("graph_snapshot").is_some();
    let preservation_artifact_identity = val.get("outputs").and_then(|v| v.as_object()).is_some();
    let mut fidelity_downgrade_reasons: Vec<String> = Vec::new();
    if !preservation_lineage {
        fidelity_downgrade_reasons.push("missing lineage".to_string());
    }
    if !preservation_run_ancestry {
        fidelity_downgrade_reasons.push("missing run ancestry".to_string());
    }
    if !preservation_graph_identity {
        fidelity_downgrade_reasons.push("missing graph identity context".to_string());
    }
    if !preservation_artifact_identity {
        fidelity_downgrade_reasons.push("missing artifact identity context".to_string());
    }

    let summary = json!({
        "bundle_version": bundle_version,
        "export_mode": val.get("export_mode").and_then(Value::as_str).unwrap_or(""),
        "verify_only": verify_only,
        "has_manifest": val.get("manifest").is_some(),
        "has_graph_snapshot": val.get("graph_snapshot").is_some(),
        "provenance_source": val
            .get("provenance")
            .and_then(|v| v.get("source"))
            .and_then(Value::as_str)
            .unwrap_or(""),
        "nodes": nodes,
        "failed_nodes": failed,
        "preservation": {
            "lineage": preservation_lineage,
            "run_ancestry": preservation_run_ancestry,
            "graph_identity": preservation_graph_identity,
            "artifact_identity": preservation_artifact_identity
        },
        "fidelity": {
            "level": if fidelity_downgrade_reasons.is_empty() {
                "exact"
            } else {
                "graded"
            },
            "downgrade_reasons": fidelity_downgrade_reasons
        },
        "invariant_violations": invariant_violations,
    });
    if !summary["invariant_violations"]
        .as_array()
        .is_some_and(|v| v.is_empty())
    {
        if cli.json {
            return emit_json(
                cli,
                "dag.import",
                false,
                summary,
                Vec::new(),
                ExitCode::from(3),
            );
        }
        println!("import summary: {}", summary);
        return Err(ExitCode::from(3));
    }
    if cli.json {
        return emit_json(
            cli,
            "dag.import",
            true,
            summary,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    } else {
        println!("import summary: {}", summary);
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{handle_export_command, handle_import_command};
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use std::path::PathBuf;

    fn quiet_json_cli() -> DagCli {
        DagCli {
            json: true,
            quiet: true,
            command: Commands::Version,
        }
    }

    #[test]
    fn export_rejects_conflicting_file_mode_flags() {
        let cli = quiet_json_cli();
        let out_dir = tempfile::tempdir().expect("tempdir");
        let code = handle_export_command(
            &cli,
            &Some(out_dir.path().to_path_buf()),
            &None,
            &out_dir.path().join("bundle.json"),
            true,
            false,
            false,
            false,
            true,
            false,
        )
        .expect_err("conflicting flags should fail");
        assert_eq!(code, ExitCode::from(2));
    }

    #[test]
    fn import_rejects_unsupported_bundle_version() {
        let cli = quiet_json_cli();
        let tmp = tempfile::NamedTempFile::new().expect("tmp file");
        std::fs::write(
            tmp.path(),
            r#"{"bundle_version":"export-bundle/v9.9","manifest":{}}"#,
        )
        .expect("write");
        let code = handle_import_command(&cli, tmp.path(), true).expect_err("unsupported version");
        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn import_accepts_verify_only_for_valid_minimal_bundle() {
        let cli = DagCli {
            json: true,
            quiet: true,
            command: Commands::Export {
                run_dir: None,
                from_run: None,
                out: PathBuf::from("unused"),
                manifest_only: true,
                without_artifacts: false,
                provenance_only: false,
                redact: false,
                with_files: false,
                include_files: false,
            },
        };
        let tmp = tempfile::NamedTempFile::new().expect("tmp file");
        std::fs::write(
            tmp.path(),
            r#"{"bundle_version":"export-bundle/v0.1","export_mode":"manifest-only","manifest":{},"graph_snapshot":{},"outputs":{},"node_traces":{},"provenance":{"source":"native-run","lineage":[],"source_run_id":"r1"}}"#,
        )
        .expect("write");
        let code = handle_import_command(&cli, tmp.path(), true).expect("valid import");
        assert_eq!(code, ExitCode::SUCCESS);
    }
}
