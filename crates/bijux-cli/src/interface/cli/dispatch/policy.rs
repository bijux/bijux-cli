//! Exit and rendering policy helpers for command execution.

use std::env;
use std::io::IsTerminal;

use crate::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use crate::interface::cli::parser::ParsedGlobalFlags;
use crate::shared::output::EmitterConfig;

fn default_output_format() -> OutputFormat {
    if std::io::stdout().is_terminal() {
        OutputFormat::Text
    } else {
        OutputFormat::Json
    }
}

pub(super) fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or_else(default_output_format),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Auto),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var_os("NO_COLOR").is_some(),
    }
}

pub(super) fn classify_error_exit_code(message: &str) -> i32 {
    let lower = message.to_ascii_lowercase();
    if lower.contains("missing argument")
        || lower.contains("invalid argument")
        || lower.contains("key cannot be empty")
        || lower.contains("invalid key")
        || lower.contains("unknown config section")
        || lower.contains("config key not found")
        || lower.contains("missing parameter")
        || lower.contains("unsupported format")
        || lower.contains("failed to load config")
        || lower.contains("unknown route:")
        || lower.contains("plugin not found")
        || lower.starts_with("invalid format:")
        || lower.starts_with("invalid color mode:")
        || lower.starts_with("invalid log level:")
    {
        2
    } else if lower.contains("non-ascii") || lower.contains("control characters") {
        3
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::classify_error_exit_code;

    #[test]
    fn classifier_maps_usage_and_encoding_messages_to_stable_codes() {
        assert_eq!(classify_error_exit_code("Missing argument: KEY required"), 2);
        assert_eq!(classify_error_exit_code("invalid format: nope"), 2);
        assert_eq!(
            classify_error_exit_code("Non-ASCII characters are not allowed in keys or values."),
            3
        );
    }
}
