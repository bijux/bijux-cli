//! Exit and rendering policy helpers for command execution.

use serde_json::Value;
use std::env;

use crate::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use crate::interface::cli::parser::ParsedGlobalFlags;
use crate::shared::output::EmitterConfig;

pub(super) fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var("NO_COLOR").ok().as_deref() == Some("1"),
    }
}

pub(super) fn classify_error_exit_code(message: &str) -> i32 {
    if message.contains("Missing argument")
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
    }
}

pub(super) fn maintenance_route_exit_code(
    normalized_path: &[String],
    payload: &Value,
) -> Option<i32> {
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
