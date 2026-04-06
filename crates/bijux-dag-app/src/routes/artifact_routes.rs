use crate::commands::DagCli;
use crate::{emit_json, inspect_artifact, ExitCode};
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::handle_artifact_inspect_command;
    use crate::commands::DagCli;
    use clap::Parser;
    use std::path::Path;

    #[test]
    fn artifact_inspect_route_rejects_missing_run_without_panic() {
        let cli = DagCli::parse_from(["dag", "artifact-inspect", "/missing/run", "n1:out"]);
        let result = handle_artifact_inspect_command(&cli, Path::new("/missing/run"), "n1:out");
        assert!(result.is_err());
    }
}
