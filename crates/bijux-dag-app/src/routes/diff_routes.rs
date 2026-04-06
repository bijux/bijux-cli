use crate::commands::DagCli;
use crate::{emit_json, print_human_diff, replay_service, ExitCode};
use std::path::Path;

pub(crate) fn handle_diff_command(
    cli: &DagCli,
    run_a: &Path,
    run_b: &Path,
    explain: bool,
    command_name: &str,
) -> Result<ExitCode, ExitCode> {
    let diff = replay_service::run_diff_from_dirs(run_a, run_b)?;
    if cli.json {
        return emit_json(
            cli,
            command_name,
            true,
            serde_json::to_value(&diff).unwrap(),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    print_human_diff(&serde_json::to_value(&diff).unwrap());
    if explain {
        println!("explain: graph fingerprint change implies cache invalidation");
        println!("explain: node fingerprint changes indicate recomputation scope");
        println!("replay_equivalent: {}", diff.replay_equivalence.equivalent);
        if !diff.replay_equivalence.reasons.is_empty() {
            println!("replay_difference_reasons: {:?}", diff.replay_equivalence.reasons);
        }
        println!("replay_reason: {}", diff.replay_equivalence.reason_report.summary);
        if !diff.replay_equivalence.cause_groups.is_empty() {
            println!(
                "replay_cause_groups: {}",
                serde_json::to_string(&diff.replay_equivalence.cause_groups).unwrap()
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}
