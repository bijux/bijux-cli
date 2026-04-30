use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stable output format contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OutputFormat {
    /// JSON output.
    Json,
    /// JSON Lines output.
    Jsonl,
    /// YAML output.
    Yaml,
    /// Human-readable text output.
    Text,
}

/// Stable pretty-print policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum PrettyMode {
    /// Pretty output enabled.
    Pretty,
    /// Compact output enabled.
    Compact,
}

/// Stable color policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ColorMode {
    /// Use terminal auto detection.
    Auto,
    /// Always emit ANSI colors.
    Always,
    /// Never emit ANSI colors.
    Never,
}

/// Stable logging level contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum LogLevel {
    /// Trace-level diagnostics.
    Trace,
    /// Debug diagnostics.
    Debug,
    /// Informational logs.
    Info,
    /// Warning logs.
    Warning,
    /// Error logs.
    Error,
    /// Critical logs.
    Critical,
}

/// Stable exit-code contract for automation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[non_exhaustive]
pub enum ExitCode {
    /// Successful execution.
    Success = 0,
    /// Internal failure.
    Error = 1,
    /// Usage or validation failure.
    Usage = 2,
    /// Encoding or serialization failure.
    Encoding = 3,
    /// User interrupt signal.
    Aborted = 130,
}

/// Stable config value source used for precedence diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ConfigSource {
    /// Value came from command-line flags.
    Flags,
    /// Value came from environment variables.
    Env,
    /// Value came from config file.
    Config,
    /// Value came from built-in defaults.
    Defaults,
}

/// Parsed global flags before precedence resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GlobalFlags {
    /// Optional output format override.
    pub output_format: Option<OutputFormat>,
    /// Optional pretty mode override.
    pub pretty_mode: Option<PrettyMode>,
    /// Optional color mode override.
    pub color_mode: Option<ColorMode>,
    /// Optional log-level override.
    pub log_level: Option<LogLevel>,
    /// Quiet mode.
    pub quiet: bool,
    /// Include runtime diagnostics.
    pub include_runtime: bool,
}

impl GlobalFlags {
    /// Build a default-empty global flag set.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            output_format: None,
            pretty_mode: None,
            color_mode: None,
            log_level: None,
            quiet: false,
            include_runtime: false,
        }
    }
}

/// Effective execution policy after precedence resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionPolicy {
    /// Effective output format.
    pub output_format: OutputFormat,
    /// Effective pretty mode.
    pub pretty_mode: PrettyMode,
    /// Effective color mode.
    pub color_mode: ColorMode,
    /// Effective log level.
    pub log_level: LogLevel,
    /// Effective quiet mode.
    pub quiet: bool,
    /// Effective runtime metadata mode.
    pub include_runtime: bool,
}

impl ExecutionPolicy {
    /// Build the baseline execution policy.
    #[must_use]
    pub fn baseline() -> Self {
        Self {
            output_format: OutputFormat::Json,
            pretty_mode: PrettyMode::Pretty,
            color_mode: ColorMode::Auto,
            log_level: LogLevel::Info,
            quiet: false,
            include_runtime: false,
        }
    }
}
