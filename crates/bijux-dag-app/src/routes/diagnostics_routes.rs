use crate::replay_service;
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
