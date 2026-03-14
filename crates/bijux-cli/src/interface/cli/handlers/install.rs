//! Product install command handler.

use std::env;
use std::io::IsTerminal;
use std::process::Command;

use anyhow::Result;
use serde_json::{json, Value};

use crate::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use crate::features::install::{install_target_aliases, resolve_install_target, Ecosystem};
use crate::interface::cli::dispatch::AppRunResult;
use crate::interface::cli::parser::ParsedGlobalFlags;
use crate::shared::argv::{command_has_flag, command_positionals};
use crate::shared::output::{render_value, EmitterConfig};

fn default_output_format() -> OutputFormat {
    if std::io::stdout().is_terminal() {
        OutputFormat::Text
    } else {
        OutputFormat::Json
    }
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or_else(default_output_format),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Auto),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var_os("NO_COLOR").is_some(),
    }
}

fn with_trailing_newline(mut content: String) -> String {
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

fn render_machine_result(
    payload: &Value,
    cfg: EmitterConfig,
    exit_code: i32,
    quiet_success: bool,
) -> Result<AppRunResult> {
    if quiet_success && exit_code == 0 {
        return Ok(AppRunResult { exit_code, stdout: String::new(), stderr: String::new() });
    }
    let rendered = with_trailing_newline(render_value(payload, cfg)?);
    Ok(if exit_code == 0 {
        AppRunResult { exit_code, stdout: rendered, stderr: String::new() }
    } else {
        AppRunResult { exit_code, stdout: String::new(), stderr: rendered }
    })
}

fn install_command_for(package_name: &str, quiet: bool) -> Vec<String> {
    let mut command = vec!["cargo".to_string(), "install".to_string(), "--locked".to_string()];
    if quiet {
        command.push("--quiet".to_string());
    }
    command.push(package_name.to_string());
    command
}

fn success_payload(
    requested_target: &str,
    canonical_target: &str,
    package_name: &str,
    executable_name: &str,
    dry_run: bool,
    command: &[String],
    stdout: &str,
    stderr: &str,
) -> Value {
    json!({
        "status": "ok",
        "requested_target": requested_target,
        "target": canonical_target,
        "installer": "cargo",
        "ecosystem": "cargo",
        "package": package_name,
        "executable": executable_name,
        "dry_run": dry_run,
        "command": command,
        "stdout": stdout,
        "stderr": stderr,
    })
}

fn error_payload(
    requested_target: &str,
    message: &str,
    exit_code: i32,
    command: Option<&[String]>,
    stdout: &str,
    stderr: &str,
) -> Value {
    json!({
        "status": "error",
        "code": exit_code,
        "requested_target": requested_target,
        "message": message,
        "installer": "cargo",
        "command": command,
        "stdout": stdout,
        "stderr": stderr,
        "supported_targets": install_target_aliases(),
    })
}

pub(crate) fn try_run(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Option<AppRunResult>> {
    if !matches!(normalized_path, [command] if command == "install") {
        return Ok(None);
    }

    let cfg = emitter_config(global_flags);
    let requested_target =
        command_positionals(argv, &["install"]).first().cloned().unwrap_or_default();
    if requested_target.trim().is_empty() {
        let payload =
            error_payload("", "Missing argument: install target is required", 2, None, "", "");
        return Ok(Some(render_machine_result(&payload, cfg, 2, false)?));
    }

    let Some(target) = resolve_install_target(&requested_target) else {
        let payload = error_payload(
            &requested_target,
            &format!("Unknown install target: {requested_target}"),
            2,
            None,
            "",
            "",
        );
        return Ok(Some(render_machine_result(&payload, cfg, 2, false)?));
    };

    let command = install_command_for(&target.strategy.package_name, global_flags.quiet);
    let dry_run = command_has_flag(argv, "--dry-run");

    if dry_run {
        let payload = success_payload(
            &requested_target,
            &target.target_name,
            &target.strategy.package_name,
            &target.strategy.executable_name,
            true,
            &command,
            "",
            "",
        );
        if cfg.format == OutputFormat::Text {
            return Ok(Some(AppRunResult {
                exit_code: 0,
                stdout: if global_flags.quiet {
                    String::new()
                } else {
                    with_trailing_newline(command.join(" "))
                },
                stderr: String::new(),
            }));
        }
        return Ok(Some(render_machine_result(&payload, cfg, 0, global_flags.quiet)?));
    }

    let output = match target.strategy.ecosystem {
        Ecosystem::Cargo => Command::new(&command[0]).args(&command[1..]).output(),
        Ecosystem::Pip => unreachable!("cargo-only install targets are supported"),
    };

    match output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

            if cfg.format == OutputFormat::Text {
                let text_stdout = if stdout.is_empty() && exit_code == 0 && stderr.is_empty() {
                    with_trailing_newline(format!(
                        "Installed `{}` with `cargo install --locked {}`.",
                        target.strategy.executable_name, target.strategy.package_name
                    ))
                } else {
                    stdout
                };
                return Ok(Some(AppRunResult {
                    exit_code,
                    stdout: if global_flags.quiet && exit_code == 0 {
                        String::new()
                    } else {
                        text_stdout
                    },
                    stderr,
                }));
            }

            let payload = if exit_code == 0 {
                success_payload(
                    &requested_target,
                    &target.target_name,
                    &target.strategy.package_name,
                    &target.strategy.executable_name,
                    false,
                    &command,
                    &stdout,
                    &stderr,
                )
            } else {
                error_payload(
                    &requested_target,
                    &format!("cargo install failed for {}", target.strategy.package_name),
                    exit_code,
                    Some(&command),
                    &stdout,
                    &stderr,
                )
            };
            Ok(Some(render_machine_result(&payload, cfg, exit_code, global_flags.quiet)?))
        }
        Err(error) => {
            if cfg.format == OutputFormat::Text {
                return Ok(Some(AppRunResult {
                    exit_code: 1,
                    stdout: String::new(),
                    stderr: with_trailing_newline(format!(
                        "failed to start `cargo install --locked {}`: {error}",
                        target.strategy.package_name
                    )),
                }));
            }

            let payload = error_payload(
                &requested_target,
                &format!(
                    "failed to start cargo install for {}: {error}",
                    target.strategy.package_name
                ),
                1,
                Some(&command),
                "",
                "",
            );
            Ok(Some(render_machine_result(&payload, cfg, 1, false)?))
        }
    }
}
