//! External runtime tool delegation helpers.

use std::process::Command;

use crate::contracts::known_bijux_tool;

use super::AppRunResult;

fn delegate_to_external_binary(
    binary: &str,
    package_name: &str,
    command_surface: &str,
    forwarded_args: &[String],
) -> AppRunResult {
    match Command::new(binary).args(forwarded_args).output() {
        Ok(output) => AppRunResult {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => {
            let message = format!(
                "failed to run `{command_surface}` via `{binary}`: {error}\ninstall with `cargo install {package_name}` or `pip install {package_name}`\n"
            );
            AppRunResult { exit_code: 1, stdout: String::new(), stderr: message }
        }
    }
}

pub(super) fn try_delegate_known_bijux_tool(argv: &[String]) -> Option<AppRunResult> {
    let first = argv.get(1)?;

    if let Some(tool) = known_bijux_tool(first) {
        let runtime_binary = tool.runtime_binary();
        let runtime_package = runtime_binary.clone();
        let command_surface = format!("bijux {}", tool.namespace);
        return Some(delegate_to_external_binary(
            &runtime_binary,
            &runtime_package,
            &command_surface,
            &argv[2..],
        ));
    }

    None
}
