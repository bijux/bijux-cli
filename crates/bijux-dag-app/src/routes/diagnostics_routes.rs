use crate::commands::DagCli;
use crate::routes::renderer::print_pretty_json;
use crate::replay_service;
use crate::emit_json;
use std::path::Path;
use std::process::ExitCode;

pub(crate) fn why_rerun_payload(run_a: &Path, run_b: &Path) -> Result<serde_json::Value, ExitCode> {
    let diff = replay_service::run_diff_from_dirs(run_a, run_b)?;
    Ok(serde_json::json!({
        "root_cause_summary": diff.replay_equivalence.reason_report.summary,
        "equivalent": diff.replay_equivalence.equivalent,
        "reasons": diff.replay_equivalence.reasons,
        "cause_groups": diff.replay_equivalence.cause_groups
    }))
}

pub(crate) fn handle_why_rerun_command(
    cli: &DagCli,
    run_a: &Path,
    run_b: &Path,
) -> Result<ExitCode, ExitCode> {
    let payload = why_rerun_payload(run_a, run_b)?;
    if cli.json {
        return emit_json(
            cli,
            "dag.why-rerun",
            true,
            payload,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    print_pretty_json(&payload);
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn trace_artifact_payload(
    run_dir: &Path,
    artifact_id: &str,
) -> Result<serde_json::Value, ExitCode> {
    let details = crate::inspect_artifact(run_dir, artifact_id)?;
    Ok(serde_json::json!({
        "artifact_id": details["artifact_id"],
        "path": details["path"],
        "provenance": details["provenance"],
        "lineage": details["lineage"]
    }))
}

pub(crate) fn handle_trace_artifact_command(
    cli: &DagCli,
    run_dir: &Path,
    artifact_id: &str,
) -> Result<ExitCode, ExitCode> {
    let payload = trace_artifact_payload(run_dir, artifact_id)?;
    if cli.json {
        return emit_json(
            cli,
            "dag.trace-artifact",
            true,
            payload,
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    print_pretty_json(&payload);
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{handle_trace_artifact_command, handle_why_rerun_command};
    use crate::commands::{Commands, DagCli};
    use crate::ExitCode;
    use std::path::{Path, PathBuf};

    fn quiet_json_cli() -> DagCli {
        DagCli {
            json: true,
            quiet: true,
            command: Commands::Version,
        }
    }

    #[test]
    fn why_rerun_route_rejects_missing_run_dir_without_panic() {
        let cli = quiet_json_cli();
        let result = handle_why_rerun_command(&cli, Path::new("/missing/a"), Path::new("/missing/b"));
        assert!(result.is_err());
    }

    #[test]
    fn trace_artifact_route_rejects_missing_run_dir_without_panic() {
        let cli = DagCli {
            json: true,
            quiet: true,
            command: Commands::TraceArtifact {
                run_dir: PathBuf::from("/missing/run"),
                artifact_id: "n1:out".to_string(),
            },
        };
        let code =
            handle_trace_artifact_command(&cli, Path::new("/missing/run"), "n1:out").unwrap_err();
        assert_eq!(code, ExitCode::from(3));
    }
}
