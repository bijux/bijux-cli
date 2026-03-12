//! Process entrypoint orchestration for the `bijux-dev-cli` executable.

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use bijux_cli::api::output::{render_value, EmitterConfig};
use bijux_cli::api::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use serde_json::{json, Value};

use crate::cli::dispatch as dev_dispatch;
use crate::runtime::query_provider::RuntimeQueryContext;

/// In-memory process output and exit result produced by the dev-cli runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn decode_os_argv() -> Result<Vec<String>, OsString> {
    let mut argv = Vec::new();
    for value in env::args_os() {
        match value.into_string() {
            Ok(valid) => argv.push(valid),
            Err(invalid) => return Err(invalid),
        }
    }
    Ok(argv)
}

fn emit_run_result(result: &AppRunResult) {
    if !result.stdout.is_empty() {
        let _ = write!(io::stdout(), "{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        let _ = write!(io::stderr(), "{}", result.stderr);
    }
}

fn synthetic_dev_cli_argv(argv: &[String]) -> Vec<String> {
    let mut synthetic = Vec::with_capacity(argv.len().saturating_add(2).max(4));
    synthetic.push("bijux".to_string());
    synthetic.push("dev".to_string());
    synthetic.push("cli".to_string());
    synthetic.extend(argv.iter().skip(1).cloned());
    synthetic
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

fn synthetic_dev_cli_parse_argv(argv: &[String]) -> Vec<String> {
    let mut globals = Vec::new();
    let mut command_tail = Vec::new();

    let mut idx = 1;
    while idx < argv.len() {
        let token = argv[idx].as_str();
        if token == "--" {
            command_tail.extend(argv.iter().skip(idx).cloned());
            break;
        }
        if is_global_flag_without_value(token) || is_global_flag_with_equals(token) {
            globals.push(argv[idx].clone());
            idx += 1;
            continue;
        }
        if is_global_flag_with_value(token) {
            globals.push(argv[idx].clone());
            if let Some(value) = argv.get(idx + 1) {
                globals.push(value.clone());
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }

        command_tail.push(argv[idx].clone());
        idx += 1;
    }

    let mut synthetic = Vec::with_capacity(1 + globals.len() + 2 + command_tail.len());
    synthetic.push("bijux".to_string());
    synthetic.extend(globals);
    synthetic.push("dev".to_string());
    synthetic.push("cli".to_string());
    synthetic.extend(command_tail);
    synthetic
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var("NO_COLOR").ok().as_deref() == Some("1"),
    }
}

fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(error.to_string())
        }
        Err(_) => None,
    }
}

fn root_help_text() -> String {
    let help_argv = vec!["bijux".to_string(), "--help".to_string()];
    try_render_clap_help(&help_argv).unwrap_or_default()
}

fn try_render_clap_result(argv: &[String]) -> Option<AppRunResult> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(AppRunResult { exit_code: 0, stdout: error.to_string(), stderr: String::new() })
        }
        Err(error) => {
            let _ = error;
            let message = root_help_text();
            let stderr = if message.ends_with('\n') { message } else { format!("{message}\n") };
            Some(AppRunResult { exit_code: 2, stdout: String::new(), stderr })
        }
    }
}

fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    if !dev_dispatch::owns_path(normalized_path) {
        return Ok(json!({"status": "error", "message": "unknown route"}));
    }

    let context = RuntimeQueryContext::from_flags(global_flags)?;
    let runtime = context.provider();
    let payload = dev_dispatch::try_handle(normalized_path, argv, &runtime)?;

    Ok(payload.unwrap_or_else(|| json!({"status": "error", "message": "unknown route"})))
}

fn maintenance_route_exit_code(normalized_path: &[String], payload: &Value) -> Option<i32> {
    let is_maintenance_runner = matches!(
        normalized_path,
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && (d == "generate" || d == "generate-all")
    ) || matches!(
        normalized_path,
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && (e == "run" || e == "run-all")
    );

    if !is_maintenance_runner {
        return None;
    }

    if payload
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status == "failed" || status == "error")
    {
        let exit_code =
            payload.get("exit_code").and_then(Value::as_i64).filter(|code| *code > 0).unwrap_or(1);
        return Some(exit_code as i32);
    }

    if payload.get("failed").and_then(Value::as_u64).is_some_and(|count| count > 0) {
        return Some(1);
    }

    if payload.get("results").and_then(Value::as_array).is_some_and(|rows| {
        rows.iter().any(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "failed" || status == "error")
        })
    }) {
        return Some(1);
    }

    Some(0)
}

/// Execute `bijux-dev-cli` and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    let synthetic_argv = synthetic_dev_cli_argv(argv);
    let synthetic_parse_argv = synthetic_dev_cli_parse_argv(argv);

    if argv.len() == 1 {
        let mut help_argv = synthetic_argv.clone();
        help_argv.push("--help".to_string());
        if let Some(help) = try_render_clap_help(&help_argv) {
            return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
        }
    }

    if let Some(result) = try_render_clap_result(&synthetic_argv) {
        return Ok(result);
    }

    let intent = parse_intent(&synthetic_parse_argv)?;
    if intent.normalized_path.is_empty() {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: root_help_text() });
    }

    let is_unknown = !dev_dispatch::owns_path(&intent.normalized_path);

    let response = route_response(&intent.normalized_path, &synthetic_argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = if message.contains("Missing argument")
                || message.contains("Invalid argument")
                || message.contains("Key cannot be empty")
                || message.contains("Invalid key")
                || message.contains("Unknown config section")
                || message.contains("Config key not found")
                || message.contains("Missing parameter")
                || message.contains("Unsupported format")
                || message.contains("Failed to load config")
            {
                2
            } else if message.contains("Non-ASCII") || message.contains("Control characters") {
                3
            } else {
                1
            };

            let rendered_error = render_value(
                &json!({
                    "status": "error",
                    "code": code,
                    "message": message,
                    "command": intent.normalized_path.join(" "),
                }),
                emitter_config(&intent.global_flags),
            )?;
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

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    let route_exit_code =
        maintenance_route_exit_code(&intent.normalized_path, &payload).unwrap_or(0);

    if intent.global_flags.quiet {
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult { exit_code: route_exit_code, stdout: content, stderr: String::new() })
}

/// Run `bijux-dev-cli` with current process argv.
#[must_use]
pub fn run_cli_from_env() -> ExitCode {
    let argv = match decode_os_argv() {
        Ok(value) => value,
        Err(_) => {
            let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
            return ExitCode::from(2);
        }
    };

    let result = match run_app(&argv) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    };

    emit_run_result(&result);
    ExitCode::from(result.exit_code as u8)
}

#[cfg(test)]
mod tests {
    use super::synthetic_dev_cli_parse_argv;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn parse_argv_lifts_global_flags_before_dev_cli_path() {
        let input = argv(&[
            "bijux-dev-cli",
            "state-audit",
            "--format",
            "text",
            "--config-path",
            "/tmp/config.env",
            "--no-pretty",
        ]);
        let synthetic = synthetic_dev_cli_parse_argv(&input);
        assert_eq!(
            synthetic,
            argv(&[
                "bijux",
                "--format",
                "text",
                "--config-path",
                "/tmp/config.env",
                "--no-pretty",
                "dev",
                "cli",
                "state-audit",
            ])
        );
    }

    #[test]
    fn parse_argv_keeps_command_options_in_tail() {
        let input = argv(&[
            "bijux-dev-cli",
            "maintenance",
            "status",
            "run",
            "--id",
            "STATUS-001",
            "--",
            "--native-flag",
        ]);
        let synthetic = synthetic_dev_cli_parse_argv(&input);
        assert_eq!(
            synthetic,
            argv(&[
                "bijux",
                "dev",
                "cli",
                "maintenance",
                "status",
                "run",
                "--id",
                "STATUS-001",
                "--",
                "--native-flag",
            ])
        );
    }
}
