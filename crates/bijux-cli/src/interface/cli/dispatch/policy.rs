//! Exit and rendering policy helpers for command execution.

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
