//! External runtime tool delegation helpers.

use std::process::Command;

use crate::contracts::known_bijux_tool;
use crate::features::apps::{resolve_control_command, resolve_runtime_command};

use super::AppRunResult;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DelegatedKnownToolCommand {
    binary: String,
    package_name: String,
    command_surface: String,
    forwarded_args: Vec<String>,
}

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

fn delegated_known_bijux_tool_command(argv: &[String]) -> Option<DelegatedKnownToolCommand> {
    match argv.get(1).map(String::as_str) {
        Some("dev") => {
            let namespace = argv.get(2)?;
            let tool = known_bijux_tool(namespace)?;
            Some(DelegatedKnownToolCommand {
                binary: resolve_control_command(namespace)
                    .map(|resolved| resolved.command)
                    .unwrap_or_else(|| tool.control_binary()),
                package_name: tool.control_package(),
                command_surface: format!("bijux dev {}", tool.namespace),
                forwarded_args: argv[3..].to_vec(),
            })
        }
        Some(namespace) => {
            let tool = known_bijux_tool(namespace)?;
            Some(DelegatedKnownToolCommand {
                binary: resolve_runtime_command(namespace)
                    .map(|resolved| resolved.command)
                    .unwrap_or_else(|| tool.runtime_binary()),
                package_name: tool.runtime_package(),
                command_surface: format!("bijux {}", tool.namespace),
                forwarded_args: argv[2..].to_vec(),
            })
        }
        None => None,
    }
}

pub(super) fn is_known_bijux_tool_route(path: &[String]) -> bool {
    match path {
        [dev, namespace, ..] => dev == "dev" && known_bijux_tool(namespace).is_some(),
        [namespace, ..] => known_bijux_tool(namespace).is_some(),
        [] => false,
    }
}

pub(super) fn delegated_command_surface(argv: &[String]) -> Option<String> {
    delegated_known_bijux_tool_command(argv).map(|command| command.command_surface)
}

pub(super) fn try_delegate_known_bijux_tool(argv: &[String]) -> Option<AppRunResult> {
    let command = delegated_known_bijux_tool_command(argv)?;
    Some(delegate_to_external_binary(
        &command.binary,
        &command.package_name,
        &command.command_surface,
        &command.forwarded_args,
    ))
}
