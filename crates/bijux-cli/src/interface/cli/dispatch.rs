//! Top-level application entrypoint and route execution.

mod delegation;
mod help;
mod policy;
mod route_exec;
mod suggest;

use anyhow::Result;
use serde_json::json;

use crate::contracts::known_bijux_tool;
use crate::interface::cli::help::render_command_help;
use crate::interface::cli::parser::parse_intent;
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::shared::output::render_value;

/// In-memory process output and exit result produced by the core app runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn root_usage_help_text() -> Result<String> {
    let help_argv = vec!["bijux".to_string(), "--help".to_string()];
    if let Some(help) = help::try_render_clap_help(&help_argv) {
        return Ok(help);
    }

    Ok(format!("{}\n", render_command_help(&[])?.trim_end()))
}

fn is_known_help_global_flag(token: &str) -> bool {
    matches!(
        token,
        "--help" | "-h" | "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text"
    )
}

fn help_global_flag_takes_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn is_known_help_global_flag_with_equals(token: &str) -> bool {
    token.starts_with("--format=")
        || token.starts_with("--log-level=")
        || token.starts_with("--color=")
        || token.starts_with("--config-path=")
}

fn parse_help_command_path(argv: &[String]) -> std::result::Result<Vec<String>, String> {
    let mut path = Vec::new();
    let mut consume_next = false;

    for token in argv.iter().skip(2) {
        if consume_next {
            consume_next = false;
            continue;
        }

        if token == "--" {
            continue;
        }
        if is_known_help_global_flag(token) || is_known_help_global_flag_with_equals(token) {
            continue;
        }
        if help_global_flag_takes_value(token) {
            consume_next = true;
            continue;
        }
        if token.starts_with('-') {
            return Err(format!("Unknown help flag: {token}"));
        }

        path.push(token.clone());
    }

    Ok(path)
}

/// Execute the CLI for provided argv and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    if argv.len() == 1 {
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&[])?.trim_end()),
            stderr: String::new(),
        });
    }

    if argv.len() == 2 && matches!(argv[1].as_str(), "--version" | "-V") {
        let normalized = vec![argv[0].clone(), "version".to_string()];
        return run_app(&normalized);
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path = match parse_help_command_path(argv) {
            Ok(path) => path,
            Err(message) => {
                let mut stderr = message;
                stderr.push('\n');
                stderr.push_str("Run `bijux --help` for available runtime commands.\n");
                return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr });
            }
        };
        let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
        if let Some(first) = path.first().map(String::as_str) {
            if first == "dev" || known_bijux_tool(first).is_some() {
                let mut delegated_argv = vec!["bijux".to_string()];
                delegated_argv.extend(path.iter().cloned());
                delegated_argv.push("--help".to_string());
                if let Some(delegated) = delegation::try_delegate_known_bijux_tool(&delegated_argv)
                {
                    return Ok(delegated);
                }
            }
        }
        let rendered = match render_command_help(&path_refs) {
            Ok(rendered) => rendered,
            Err(_) => {
                return Ok(AppRunResult {
                    exit_code: 2,
                    stdout: String::new(),
                    stderr:
                        "Unknown help topic. Run `bijux --help` for available runtime commands.\n"
                            .to_string(),
                });
            }
        };
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", rendered.trim_end()),
            stderr: String::new(),
        });
    }

    let has_help_flag = argv.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"));
    if has_help_flag
        && argv.get(1).is_some_and(|first| first == "dev" || known_bijux_tool(first).is_some())
    {
        if let Some(delegated) = delegation::try_delegate_known_bijux_tool(argv) {
            return Ok(delegated);
        }
    }

    if let Some(help) = help::try_render_clap_help(argv) {
        return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
    }

    if let Some(delegated) = delegation::try_delegate_known_bijux_tool(argv) {
        return Ok(delegated);
    }

    if let Some(usage_error) = help::try_render_clap_usage_error(argv) {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: if usage_error.ends_with('\n') {
                usage_error
            } else {
                format!("{usage_error}\n")
            },
        });
    }

    let intent = parse_intent(argv)?;
    if intent.normalized_path.is_empty() {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: root_usage_help_text()?,
        });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let response = route_exec::route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = policy::classify_error_exit_code(&message);
            let mut error_payload = json!({
                "status": "error",
                "code": code,
                "message": message,
                "command": intent.normalized_path.join(" "),
            });
            if message.starts_with("unknown route: ") {
                if let Some(correction) =
                    suggest::correction_for_unknown_route(&intent.normalized_path)
                {
                    let nearest_command = correction.nearest_command;
                    let next_command = correction.next_command;
                    let next_help = correction.next_help;
                    error_payload["nearest_command"] = json!(nearest_command);
                    error_payload["next_command"] = json!(next_command.clone());
                    error_payload["next_help"] = json!(next_help.clone());
                    error_payload["hint"] =
                        json!(format!("Try `{}` or `{}`.", next_command, next_help));
                }
            }
            let rendered_error =
                render_value(&error_payload, policy::emitter_config(&intent.global_flags))?;
            let error_content = if rendered_error.ends_with('\n') {
                rendered_error
            } else {
                format!("{rendered_error}\n")
            };
            return Ok(AppRunResult {
                exit_code: code,
                stdout: String::new(),
                stderr: error_content,
            });
        }
    };

    let rendered = render_value(&payload, policy::emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    let route_exit_code = 0;

    if intent.global_flags.quiet {
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult { exit_code: route_exit_code, stdout: content, stderr: String::new() })
}
