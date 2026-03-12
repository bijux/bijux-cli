//! External tool and control-plane delegation helpers.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::contracts::known_bijux_tool;

use super::AppRunResult;

const DEV_CLI_BINARY: &str = "bijux-dev-cli";
const DEV_CLI_PACKAGE: &str = "bijux-dev-cli";

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
            AppRunResult {
                exit_code: 1,
                stdout: String::new(),
                stderr: message,
            }
        }
    }
}

fn executable_name(binary: &str) -> String {
    let extension = env::consts::EXE_EXTENSION;
    if extension.is_empty() {
        binary.to_string()
    } else {
        format!("{binary}.{extension}")
    }
}

fn push_unique_candidate(candidates: &mut Vec<String>, path: PathBuf) {
    let value = path.to_string_lossy().into_owned();
    if !candidates.iter().any(|candidate| candidate == &value) {
        candidates.push(value);
    }
}

fn dev_cli_binary_candidates() -> Vec<String> {
    if let Ok(explicit) = env::var("BIJUX_DEV_CLI_BIN") {
        return vec![explicit];
    }

    let mut candidates = Vec::new();
    let executable = executable_name(DEV_CLI_BINARY);

    if let Ok(current_exe) = env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            push_unique_candidate(&mut candidates, dir.join(&executable));
            if dir.file_name().is_some_and(|name| name == "deps") {
                if let Some(parent) = dir.parent() {
                    push_unique_candidate(&mut candidates, parent.join(&executable));
                }
            }
        }
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(workspace_root) = manifest_dir.parent().and_then(Path::parent) {
        for profile in ["debug", "release"] {
            push_unique_candidate(
                &mut candidates,
                workspace_root
                    .join("target")
                    .join(profile)
                    .join(&executable),
            );
        }
    }

    // Keep PATH lookup as the final fallback so workspace-local binaries win in tests and CI.
    candidates.push(DEV_CLI_BINARY.to_string());

    candidates
}

fn delegate_dev_cli(forwarded_args: &[String]) -> AppRunResult {
    let candidates = dev_cli_binary_candidates();
    let mut last_error = String::new();
    let mut fallback_usage: Option<AppRunResult> = None;

    for (index, binary) in candidates.iter().enumerate() {
        match Command::new(binary).args(forwarded_args).output() {
            Ok(output) => {
                let result = AppRunResult {
                    exit_code: output.status.code().unwrap_or(1),
                    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                };

                let diagnostic_stream = if result.stderr.trim().is_empty() {
                    result.stdout.as_str()
                } else {
                    result.stderr.as_str()
                };
                let looks_like_generic_root_help = result.exit_code != 0
                    && diagnostic_stream.contains("Usage: bijux [OPTIONS] [COMMAND]")
                    && diagnostic_stream.contains("Commands:");

                if looks_like_generic_root_help && index + 1 < candidates.len() {
                    fallback_usage = Some(result);
                    continue;
                }

                return result;
            }
            Err(error) => {
                last_error = format!("{binary}: {error}");
            }
        }
    }

    if let Some(result) = fallback_usage {
        return result;
    }

    let attempted = candidates.join(", ");
    let message = format!(
        "failed to run `bijux dev cli`: {last_error}\nattempted binaries: {attempted}\ninstall with `cargo install {DEV_CLI_PACKAGE}` or `pip install {DEV_CLI_PACKAGE}`\n"
    );
    AppRunResult {
        exit_code: 1,
        stdout: String::new(),
        stderr: message,
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

    if first == "dev" {
        let Some(tool_namespace) = argv.get(2) else {
            return Some(delegate_dev_cli(&[]));
        };
        if let Some(tool) = known_bijux_tool(tool_namespace) {
            let control_binary = tool.control_binary();
            let control_package = control_binary.clone();
            let command_surface = format!("bijux dev {}", tool.namespace);
            return Some(delegate_to_external_binary(
                &control_binary,
                &control_package,
                &command_surface,
                &argv[3..],
            ));
        }

        let forwarded = if tool_namespace == "cli" {
            &argv[3..]
        } else {
            &argv[2..]
        };
        return Some(delegate_dev_cli(forwarded));
    }

    None
}
