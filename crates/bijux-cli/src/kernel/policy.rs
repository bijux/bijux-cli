#![forbid(unsafe_code)]
//! Kernel policy resolution utilities.

use crate::routing::{ColorMode, ExecutionPolicy, GlobalFlags, LogLevel, OutputFormat, PrettyMode};

use super::pipeline::ExecutionIntent;

/// Layered policy inputs from env/config/defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyInputs {
    /// Environment-provided policy values.
    pub env: GlobalFlags,
    /// Config-provided policy values.
    pub config: GlobalFlags,
    /// Hard defaults.
    pub defaults: GlobalFlags,
}

fn choose_opt<T: Copy>(a: Option<T>, b: Option<T>, c: Option<T>) -> Option<T> {
    a.or(b).or(c)
}

/// Resolve effective policy using flags -> env -> config -> defaults precedence.
#[must_use]
pub fn resolve_policy(intent: &ExecutionIntent, inputs: &PolicyInputs) -> ExecutionPolicy {
    let output_format = choose_opt(
        intent.global_flags.output_format,
        inputs.env.output_format,
        choose_opt(
            inputs.config.output_format,
            inputs.defaults.output_format,
            None,
        ),
    )
    .unwrap_or(OutputFormat::Json);

    let pretty_mode = choose_opt(
        intent.global_flags.pretty_mode,
        inputs.env.pretty_mode,
        choose_opt(inputs.config.pretty_mode, inputs.defaults.pretty_mode, None),
    )
    .unwrap_or(PrettyMode::Pretty);

    let color_mode = choose_opt(
        intent.global_flags.color_mode,
        inputs.env.color_mode,
        choose_opt(inputs.config.color_mode, inputs.defaults.color_mode, None),
    )
    .unwrap_or(ColorMode::Auto);

    let mut log_level = choose_opt(
        intent.global_flags.log_level,
        inputs.env.log_level,
        choose_opt(inputs.config.log_level, inputs.defaults.log_level, None),
    )
    .unwrap_or(LogLevel::Info);

    let quiet = intent.global_flags.quiet || inputs.env.quiet || inputs.config.quiet;
    if quiet {
        log_level = LogLevel::Error;
    }

    ExecutionPolicy {
        output_format,
        pretty_mode,
        color_mode,
        log_level,
        quiet,
        include_runtime: intent.global_flags.include_runtime || inputs.env.include_runtime,
    }
}
