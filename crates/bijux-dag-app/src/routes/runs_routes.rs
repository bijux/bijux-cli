use crate::commands::{DagCli, RunsCommands};
use crate::inspect_service;
use crate::routes::selector_grammar::parse_selector_expressions;
use crate::{
    emit_json, format_inspect_human, format_show_human, list_runs, print_human_diff,
    replay_service, resolve_run_dir, runs_compare, runs_failures, runs_flakes, runs_summary,
    runs_trend, verify_run, ExitCode,
};

pub(crate) fn handle_runs_command(
    cli: &DagCli,
    command: &RunsCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        RunsCommands::List { root } => {
            let runs = list_runs(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.list",
                    true,
                    serde_json::json!({"runs": runs}),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for run in runs {
                println!("{run}");
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Show { run_id, root } => {
            let summary = inspect_service::run_summary_for_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.show",
                    true,
                    summary,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_show_human(&summary));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Inspect { run_id, root } => {
            let summary = inspect_service::run_summary_for_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.inspect",
                    true,
                    summary,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", format_inspect_human(&summary));
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::History { root, status, source, offset, limit, select } => {
            let selectors = parse_selector_expressions(select)?;
            let pagination = limit.map(|value| (offset.unwrap_or(0), value));
            let report = inspect_service::run_history_query_for_root(
                root,
                status.as_deref(),
                source.as_deref(),
                pagination,
                Some(selectors.as_slice()),
            )?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.history",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::IdExplain { run_id, root } => {
            let report = inspect_service::run_id_explain_for_root(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.id-explain",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Tree { run_id, root } => {
            let tree = inspect_service::run_tree_for_id(root, run_id)?;
            if cli.json {
                return emit_json(cli, "dag.runs.tree", true, tree, Vec::new(), ExitCode::SUCCESS);
            }
            println!("{}", serde_json::to_string_pretty(&tree).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Timeline { run_id, root } => {
            let timeline = inspect_service::run_timeline_for_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.timeline",
                    true,
                    timeline,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&timeline).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Diff { run_a, run_b, mode, node, explain } => {
            let payload =
                replay_service::run_diff_mode_payload(run_a, run_b, *mode, node.as_deref())?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.diff",
                    true,
                    payload.clone(),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if matches!(mode, crate::commands::DiffModeArg::Semantic) {
                print_human_diff(&payload);
            } else {
                println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            }
            if *explain {
                if let Some(summary) = payload
                    .get("root_cause_summary")
                    .or_else(|| {
                        payload
                            .get("replay_equivalence")
                            .and_then(|v| v.get("reason_report"))
                            .and_then(|v| v.get("summary"))
                    })
                    .and_then(serde_json::Value::as_str)
                {
                    println!("replay_reason: {summary}");
                }
                if let Some(cause_groups) = payload.get("cause_groups").or_else(|| {
                    payload.get("replay_equivalence").and_then(|v| v.get("cause_groups"))
                }) {
                    println!(
                        "replay_cause_groups: {}",
                        serde_json::to_string(cause_groups).unwrap()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Verify { run_id, root, deep, strict } => {
            let run_dir = resolve_run_dir(root, run_id);
            let report = verify_run(&run_dir, *deep, *strict)?;
            let ok =
                report.get("status").and_then(|v| v.as_str()).map(|v| v == "ok").unwrap_or(false);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.verify",
                    ok,
                    report,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("status: {}", if ok { "ok" } else { "invalid" });
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Doctor { run_id, root } => {
            let report = inspect_service::doctor_for_run_id(root, run_id);
            let ok = report.get("status").and_then(|v| v.as_str()) == Some("ok");
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.doctor",
                    ok,
                    report,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if !ok {
                return Err(ExitCode::from(3));
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::ExplainFailure { run_id, root } => {
            let report = inspect_service::explain_failure_for_run_id(root, run_id)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.explain-failure",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Summary { root } => {
            let report = runs_summary(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.summary",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Compare { run_a, run_b, root } => {
            let report = runs_compare(root, run_a, run_b).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.compare",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Trend { root } => {
            let report = runs_trend(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.trend",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Failures { root } => {
            let report = runs_failures(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.failures",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Flakes { root } => {
            let report = runs_flakes(root).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.flakes",
                    true,
                    report,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_runs_command;
    use crate::commands::{Commands, DagCli, RunsCommands};
    use crate::ExitCode;
    use serde_json::json;
    use std::fs;
    use std::path::Path;

    fn quiet_json_cli() -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Version }
    }

    fn write_run(root: &Path, run_id: &str, imported: bool) {
        let run = root.join(run_id);
        fs::create_dir_all(run.join("nodes/n1")).expect("mkdir nodes");
        let mut manifest = json!({
            "run_id": run_id,
            "status": "success",
            "run_dir_format": "run-dir/v0.1",
            "graph_fingerprint": "g1",
            "created_unix_ms": 1,
            "started_unix_ms": 1,
            "finished_unix_ms": 2,
            "node_counts": {"success": 1, "failed": 0, "skipped": 0, "cached": 0},
            "run_metadata": {"submission_source": "manual", "trigger_source": "manual", "labels": ["etl"]}
        });
        if imported {
            manifest["run_metadata"]["submission_source"] = json!("imported");
            manifest["run_metadata"]["labels"] = json!(["etl", "imported"]);
        }
        fs::write(
            run.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            run.join("snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "nodes": [{"id":"n1"}],
                    "edges": []
                }
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(run.join("outputs.index.json"), b"[]").expect("write outputs index");
        fs::write(
            run.join("nodes/n1/trace.json"),
            serde_json::to_vec_pretty(&json!({
                "status":"success","started_unix_ms":1,"finished_unix_ms":2,"attempt":1
            }))
            .expect("trace"),
        )
        .expect("write trace");
    }

    #[test]
    fn runs_routes_support_listing_and_summary_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-a", false);
        let cli = quiet_json_cli();
        let list =
            handle_runs_command(&cli, &RunsCommands::List { root: tmp.path().to_path_buf() })
                .expect("list");
        assert_eq!(list, ExitCode::SUCCESS);
        let summary =
            handle_runs_command(&cli, &RunsCommands::Summary { root: tmp.path().to_path_buf() })
                .expect("summary");
        assert_eq!(summary, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_support_timeline_and_tree_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-tree", false);
        let cli = quiet_json_cli();
        let tree = handle_runs_command(
            &cli,
            &RunsCommands::Tree { run_id: "run-tree".to_string(), root: tmp.path().to_path_buf() },
        )
        .expect("tree");
        assert_eq!(tree, ExitCode::SUCCESS);
        let timeline = handle_runs_command(
            &cli,
            &RunsCommands::Timeline {
                run_id: "run-tree".to_string(),
                root: tmp.path().to_path_buf(),
            },
        )
        .expect("timeline");
        assert_eq!(timeline, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_support_imported_bundle_like_flows() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-imported", true);
        let cli = quiet_json_cli();
        let inspect = handle_runs_command(
            &cli,
            &RunsCommands::Inspect {
                run_id: "run-imported".to_string(),
                root: tmp.path().to_path_buf(),
            },
        )
        .expect("inspect");
        assert_eq!(inspect, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_history_supports_filter_and_pagination_flags() {
        let tmp = tempfile::tempdir().expect("tmp");
        write_run(tmp.path(), "run-a", false);
        write_run(tmp.path(), "run-b", true);
        let cli = quiet_json_cli();
        let history = handle_runs_command(
            &cli,
            &RunsCommands::History {
                root: tmp.path().to_path_buf(),
                status: Some("success".to_string()),
                source: Some("imported".to_string()),
                offset: Some(0),
                limit: Some(10),
                select: vec!["tag:etl".to_string(), "run:run-b".to_string()],
            },
        )
        .expect("history");
        assert_eq!(history, ExitCode::SUCCESS);
    }

    #[test]
    fn runs_routes_tolerate_corrupted_run_dir_without_panic() {
        let tmp = tempfile::tempdir().expect("tmp");
        let run = tmp.path().join("run-bad");
        fs::create_dir_all(&run).expect("mkdir");
        fs::write(run.join("manifest.json"), b"{bad-json").expect("manifest");
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| {
            handle_runs_command(
                &cli,
                &RunsCommands::Timeline {
                    run_id: "run-bad".to_string(),
                    root: tmp.path().to_path_buf(),
                },
            )
        });
        assert!(result.is_ok(), "timeline flow should not panic");
        assert!(result.expect("result").is_ok());
    }
}
