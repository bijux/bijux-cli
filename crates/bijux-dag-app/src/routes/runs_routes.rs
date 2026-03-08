use crate::commands::{DagCli, RunsCommands};
use crate::inspect_service;
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
        RunsCommands::History { root } => {
            let report = inspect_service::run_history_for_root(root)?;
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
                return emit_json(
                    cli,
                    "dag.runs.tree",
                    true,
                    tree,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
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
        RunsCommands::Diff {
            run_a,
            run_b,
            mode: _mode,
            explain,
        } => {
            let diff = replay_service::run_diff_from_dirs(run_a, run_b)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.diff",
                    true,
                    serde_json::to_value(&diff).unwrap(),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            print_human_diff(&serde_json::to_value(&diff).unwrap());
            if *explain {
                println!("explain: graph fingerprint change implies cache invalidation");
                println!(
                    "replay_reason: {}",
                    diff.replay_equivalence.reason_report.summary
                );
                if !diff.replay_equivalence.cause_groups.is_empty() {
                    println!(
                        "replay_cause_groups: {}",
                        serde_json::to_string(&diff.replay_equivalence.cause_groups).unwrap()
                    );
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        RunsCommands::Verify {
            run_id,
            root,
            deep,
            strict,
        } => {
            let run_dir = resolve_run_dir(root, run_id);
            let report = verify_run(&run_dir, *deep, *strict)?;
            let ok = report
                .get("status")
                .and_then(|v| v.as_str())
                .map(|v| v == "ok")
                .unwrap_or(false);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runs.verify",
                    ok,
                    report,
                    Vec::new(),
                    if ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
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
                    if ok {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
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
