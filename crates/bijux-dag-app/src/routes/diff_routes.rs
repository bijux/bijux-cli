use crate::commands::{DagCli, DiffModeArg};
use crate::{emit_json, print_human_diff, replay_service, ExitCode};
use std::path::Path;

pub(crate) fn handle_diff_command(
    cli: &DagCli,
    run_a: &Path,
    run_b: &Path,
    mode: DiffModeArg,
    node: Option<&str>,
    explain: bool,
    command_name: &str,
) -> Result<ExitCode, ExitCode> {
    let payload = replay_service::run_diff_mode_payload(run_a, run_b, mode, node)?;
    if cli.json {
        return emit_json(cli, command_name, true, payload.clone(), Vec::new(), ExitCode::SUCCESS);
    }
    if matches!(mode, DiffModeArg::Semantic) {
        print_human_diff(&payload);
    } else {
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
    if explain {
        if matches!(mode, DiffModeArg::Semantic | DiffModeArg::Summary | DiffModeArg::Raw) {
            let semantic = if matches!(mode, DiffModeArg::Semantic) {
                Some(&payload)
            } else {
                payload.get("semantic").or(Some(&payload))
            };
            if let Some(semantic) = semantic {
                let replay_equivalent = semantic
                    .get("replay_equivalence")
                    .and_then(|value| value.get("equivalent"))
                    .and_then(serde_json::Value::as_bool);
                let reasons = semantic
                    .get("replay_equivalence")
                    .and_then(|value| value.get("reasons"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let cause_groups = semantic
                    .get("replay_equivalence")
                    .and_then(|value| value.get("cause_groups"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let summary = payload.get("root_cause_summary").or_else(|| {
                    semantic
                        .get("replay_equivalence")
                        .and_then(|value| value.get("reason_report"))
                        .and_then(|value| value.get("summary"))
                });
                println!("replay_equivalent: {}", replay_equivalent.unwrap_or(false));
                if reasons.is_array() && reasons.as_array().is_some_and(|items| !items.is_empty()) {
                    println!("replay_difference_reasons: {}", reasons);
                }
                if let Some(summary) = summary.and_then(serde_json::Value::as_str) {
                    println!("replay_reason: {summary}");
                }
                if cause_groups.is_object()
                    && cause_groups.as_object().is_some_and(|map| !map.is_empty())
                {
                    println!("replay_cause_groups: {}", cause_groups);
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
