use crate::commands::DagCli;
use crate::routes::replay_routes;
use crate::{emit_json, verify_bundle_invariants, verify_run, ExitCode};
use serde_json::json;
use std::path::Path;

pub(crate) fn handle_prove_command(cli: &DagCli, run_dir: &Path) -> Result<ExitCode, ExitCode> {
    replay_routes::handle_prove_command(cli, run_dir)
}

pub(crate) fn handle_proof_summary_command(
    cli: &DagCli,
    run_dir: &Path,
) -> Result<ExitCode, ExitCode> {
    replay_routes::handle_proof_summary_command(cli, run_dir)
}

pub(crate) fn handle_verify_command(
    cli: &DagCli,
    run_dir: &Path,
    deep: bool,
    strict: bool,
) -> Result<ExitCode, ExitCode> {
    let report = verify_run(run_dir, deep, strict)?;
    let ok = report
        .get("status")
        .and_then(|v| v.as_str())
        .map(|v| v == "ok")
        .unwrap_or(false);
    if cli.json {
        return emit_json(
            cli,
            "dag.verify",
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
    println!("status: {}", report["status"]);
    if !ok {
        return Err(ExitCode::from(3));
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn handle_fsck_command(
    cli: &DagCli,
    run_dir: &Path,
    strict: bool,
) -> Result<ExitCode, ExitCode> {
    let (report, ok) = if run_dir.is_file() {
        let data = crate::read_file(run_dir)?;
        let val: serde_json::Value = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
        let violations = verify_bundle_invariants(&val);
        let ok = violations.is_empty();
        (
            json!({
                "kind": "bundle",
                "path": run_dir,
                "status": if ok { "ok" } else { "error" },
                "invariant_violations": violations
            }),
            ok,
        )
    } else {
        let report = verify_run(run_dir, true, strict)?;
        let ok = report
            .get("status")
            .and_then(|v| v.as_str())
            .map(|v| v == "ok")
            .unwrap_or(false);
        (report, ok)
    };
    if cli.json {
        return emit_json(
            cli,
            "dag.fsck",
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
    println!("status: {}", report["status"]);
    if !ok {
        return Err(ExitCode::from(3));
    }
    Ok(ExitCode::SUCCESS)
}
