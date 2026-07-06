//! Official app delegation helpers across binary, Python, and embedded mounts.

use std::process::Command;

use serde_json::json;

use crate::contracts::{known_bijux_tool_by_query, KnownBijuxTool, ProductEntrypointKind};
use crate::features::apps::{resolve_control_command, resolve_runtime_command, ResolvedAppCommand};

use super::AppRunResult;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DelegatedKnownToolCommand {
    resolved: ResolvedAppCommand,
    install_commands: Vec<String>,
    public_help_binary: Option<String>,
    command_surface: String,
    forwarded_args: Vec<String>,
}

fn is_global_flag_without_value(token: &str) -> bool {
    matches!(token, "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text")
}

fn is_global_flag_with_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn is_global_flag_with_equals(token: &str) -> bool {
    token.starts_with("--format=")
        || token.starts_with("--log-level=")
        || token.starts_with("--color=")
        || token.starts_with("--config-path=")
}

fn skip_root_globals(argv: &[String]) -> usize {
    let mut idx = 1;
    while idx < argv.len() {
        let token = argv[idx].as_str();
        if is_global_flag_without_value(token) || is_global_flag_with_equals(token) {
            idx += 1;
            continue;
        }
        if is_global_flag_with_value(token) {
            idx += 2;
            continue;
        }
        break;
    }
    idx
}

fn locate_known_tool_route(argv: &[String]) -> Option<(bool, String, usize)> {
    let command_start = skip_root_globals(argv);
    let first = argv.get(command_start)?;
    if first == "dev" {
        let query = argv.get(command_start + 1)?;
        known_bijux_tool_by_query(query).map(|_| (true, query.clone(), command_start + 2))
    } else {
        if known_bijux_tool_by_query(first).is_some() || resolve_runtime_command(first).is_some() {
            Some((false, first.clone(), command_start + 1))
        } else {
            None
        }
    }
}

fn render_embedded_descriptor_help(command_surface: &str, resolved: &ResolvedAppCommand) -> String {
    let aliases =
        resolved.descriptor.aliases.iter().map(|alias| alias.0.as_str()).collect::<Vec<_>>();
    let alias_line = if aliases.is_empty() {
        String::new()
    } else {
        format!("\nAliases:\n  {}\n", aliases.join(", "))
    };
    let version_line = resolved
        .descriptor
        .version
        .as_ref()
        .map(|value| format!("\nVersion:\n  {value}\n"))
        .unwrap_or_default();
    let capabilities = if resolved.descriptor.capabilities.is_empty() {
        "  (none)\n".to_string()
    } else {
        format!("  {}\n", resolved.descriptor.capabilities.join(", "))
    };

    format!(
        "Usage: {command_surface} [status|version|--help]\n\n{}\n{}\nCapabilities:\n{}{}",
        resolved.descriptor.help.summary,
        alias_line.trim_end(),
        capabilities,
        version_line
    )
}

fn run_embedded_descriptor_shell(
    resolved: &ResolvedAppCommand,
    command_surface: &str,
    forwarded_args: &[String],
) -> AppRunResult {
    let first = forwarded_args.first().map(String::as_str);
    if forwarded_args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"))
        || matches!(first, Some("help"))
    {
        return AppRunResult {
            exit_code: 0,
            stdout: format!(
                "{}\n",
                render_embedded_descriptor_help(command_surface, resolved).trim_end()
            ),
            stderr: String::new(),
        };
    }

    if matches!(first, Some("version")) {
        let version = resolved
            .descriptor
            .version
            .clone()
            .unwrap_or_else(|| format!("{} embedded", resolved.namespace));
        return AppRunResult {
            exit_code: 0,
            stdout: format!("{version}\n"),
            stderr: String::new(),
        };
    }

    if forwarded_args.is_empty() || matches!(first, Some("status")) {
        let payload = json!({
            "status": "ok",
            "namespace": resolved.namespace,
            "mode": "embedded_rust",
            "handler": resolved.command,
            "summary": resolved.descriptor.help.summary,
            "capabilities": resolved.descriptor.capabilities,
        });
        return AppRunResult {
            exit_code: 0,
            stdout: format!(
                "{}\n",
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string())
            ),
            stderr: String::new(),
        };
    }

    AppRunResult {
        exit_code: 2,
        stdout: String::new(),
        stderr: format!(
            "embedded handler `{}` for `{command_surface}` does not support `{}`\n",
            resolved.command,
            forwarded_args.join(" ")
        ),
    }
}

fn delegate_to_embedded_handler(
    resolved: &ResolvedAppCommand,
    command_surface: &str,
    forwarded_args: &[String],
) -> AppRunResult {
    match resolved.command.as_str() {
        "descriptor-shell" => {
            run_embedded_descriptor_shell(resolved, command_surface, forwarded_args)
        }
        other => AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!(
                "embedded handler `{other}` for `{command_surface}` is not registered\n"
            ),
        },
    }
}

fn delegate_to_resolved_command(
    resolved: &ResolvedAppCommand,
    install_commands: &[String],
    public_help_binary: Option<&str>,
    command_surface: &str,
    forwarded_args: &[String],
) -> AppRunResult {
    if matches!(resolved.kind, ProductEntrypointKind::EmbeddedRust) {
        return delegate_to_embedded_handler(resolved, command_surface, forwarded_args);
    }

    match Command::new(&resolved.command).args(&resolved.args).args(forwarded_args).output() {
        Ok(output) => AppRunResult {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => {
            let mut message = format!(
                "failed to run `{command_surface}` via `{}`: {error}\n",
                resolved.display_command
            );
            if !install_commands.is_empty() {
                message.push_str("install with:\n");
                for command in install_commands {
                    message.push_str(&format!("  {command}\n"));
                }
            }
            if let Some(public_help_binary) = public_help_binary {
                message.push_str(&format!(
                    "use `{public_help_binary} --help` for the public command surface\n"
                ));
            }
            AppRunResult { exit_code: 1, stdout: String::new(), stderr: message }
        }
    }
}

fn install_commands_for_known_tool(tool: &KnownBijuxTool, control_plane: bool) -> Vec<String> {
    let package_name =
        if control_plane { tool.control_package_name } else { tool.runtime_package_name };
    match tool.language {
        "python" => vec![format!("pip install {package_name}")],
        _ => vec![format!("cargo install {package_name}")],
    }
}

fn delegated_known_bijux_tool_command(argv: &[String]) -> Option<DelegatedKnownToolCommand> {
    let (control_plane, query, forwarded_start) = locate_known_tool_route(argv)?;

    if control_plane {
        let tool = known_bijux_tool_by_query(&query)?;
        Some(DelegatedKnownToolCommand {
            resolved: resolve_control_command(&query).unwrap_or_else(|| ResolvedAppCommand {
                command: tool.control_binary(),
                args: Vec::new(),
                display_command: tool.control_binary(),
                source: crate::features::apps::AppDiscoverySource::CompiledOfficialRegistry,
                kind: ProductEntrypointKind::Binary,
                namespace: tool.namespace.to_string(),
                descriptor: tool.descriptor(),
            }),
            install_commands: install_commands_for_known_tool(tool, true),
            public_help_binary: None,
            command_surface: format!("bijux dev {}", tool.namespace),
            forwarded_args: argv[forwarded_start..].to_vec(),
        })
    } else {
        let tool = known_bijux_tool_by_query(&query);
        let resolved = if let Some(tool) = known_bijux_tool_by_query(&query) {
            resolve_runtime_command(&query).unwrap_or_else(|| ResolvedAppCommand {
                command: tool.runtime_binary(),
                args: Vec::new(),
                display_command: tool.runtime_binary(),
                source: crate::features::apps::AppDiscoverySource::CompiledOfficialRegistry,
                kind: ProductEntrypointKind::Binary,
                namespace: tool.namespace.to_string(),
                descriptor: tool.descriptor(),
            })
        } else {
            resolve_runtime_command(&query)?
        };
        Some(DelegatedKnownToolCommand {
            command_surface: format!("bijux {}", resolved.namespace),
            install_commands: tool
                .map(|tool| install_commands_for_known_tool(tool, false))
                .unwrap_or_default(),
            public_help_binary: tool.map(|tool| tool.runtime_binary_name.to_string()),
            resolved,
            forwarded_args: argv[forwarded_start..].to_vec(),
        })
    }
}

pub(super) fn is_known_bijux_tool_route(path: &[String]) -> bool {
    match path {
        [dev, namespace, ..] => dev == "dev" && known_bijux_tool_by_query(namespace).is_some(),
        [namespace, ..] => resolve_runtime_command(namespace).is_some(),
        [] => false,
    }
}

pub(super) fn delegated_command_surface(argv: &[String]) -> Option<String> {
    delegated_known_bijux_tool_command(argv).map(|command| command.command_surface)
}

pub(super) fn try_delegate_known_bijux_tool(argv: &[String]) -> Option<AppRunResult> {
    let command = delegated_known_bijux_tool_command(argv)?;
    Some(delegate_to_resolved_command(
        &command.resolved,
        &command.install_commands,
        command.public_help_binary.as_deref(),
        &command.command_surface,
        &command.forwarded_args,
    ))
}
