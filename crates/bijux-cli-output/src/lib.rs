#![forbid(unsafe_code)]
//! Output encoding and envelope rendering surfaces.

use bijux_cli_contracts::{
    ColorMode, ContractMarker, ErrorEnvelopeV1, ErrorPayloadV1, LogLevel, OutputEnvelopeV1,
    OutputFormat,
};
use serde_json::{json, Value};

/// Output stream target for emitters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// Standard output stream.
    Stdout,
    /// Standard error stream.
    Stderr,
}

/// Rendered output payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedOutput {
    /// Output stream target.
    pub stream: OutputStream,
    /// Rendered content.
    pub content: String,
}

/// Emitter configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitterConfig {
    /// Render format.
    pub format: OutputFormat,
    /// Pretty rendering toggle.
    pub pretty: bool,
    /// Color mode policy.
    pub color: ColorMode,
    /// Log-level formatting control.
    pub log_level: LogLevel,
    /// Quiet mode suppression.
    pub quiet: bool,
    /// External no-color policy flag.
    pub no_color: bool,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Json,
            pretty: true,
            color: ColorMode::Auto,
            log_level: LogLevel::Info,
            quiet: false,
            no_color: false,
        }
    }
}

/// Emitter-level errors.
#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    /// JSON serialization failed.
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    /// YAML serialization failed.
    #[error("yaml serialization failed: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

fn should_emit_color(cfg: EmitterConfig) -> bool {
    if cfg.no_color {
        return false;
    }

    match cfg.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => true,
        _ => true,
    }
}

fn colorize_error(s: &str, cfg: EmitterConfig) -> String {
    if should_emit_color(cfg) {
        format!("\u{001b}[31m{s}\u{001b}[0m")
    } else {
        s.to_string()
    }
}

/// Render arbitrary value in configured format.
pub fn render_value(value: &Value, cfg: EmitterConfig) -> Result<String, EmitError> {
    match cfg.format {
        OutputFormat::Json => {
            if cfg.pretty {
                serde_json::to_string_pretty(value).map_err(EmitError::from)
            } else {
                serde_json::to_string(value).map_err(EmitError::from)
            }
        }
        OutputFormat::Yaml => serde_yaml::to_string(value).map_err(EmitError::from),
        OutputFormat::Text => {
            if let Some(text) = value.as_str() {
                Ok(text.to_string())
            } else if cfg.pretty {
                serde_json::to_string_pretty(value).map_err(EmitError::from)
            } else {
                serde_json::to_string(value).map_err(EmitError::from)
            }
        }
        _ => serde_json::to_string(value).map_err(EmitError::from),
    }
}

/// Render success envelope to stdout, honoring quiet mode rules.
pub fn emit_success(
    envelope: &OutputEnvelopeV1,
    cfg: EmitterConfig,
) -> Result<Option<RenderedOutput>, EmitError> {
    if cfg.quiet && cfg.format == OutputFormat::Text {
        return Ok(None);
    }

    let value = serde_json::to_value(envelope)?;
    let mut content = render_value(&value, cfg)?;
    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(Some(RenderedOutput { stream: OutputStream::Stdout, content }))
}

/// Build machine-safe error payload preserving stable fields.
#[must_use]
pub fn machine_safe_error_payload(payload: &ErrorPayloadV1) -> Value {
    json!({
        "code": payload.code,
        "message": payload.message,
        "category": payload.category,
        "details": payload.details,
    })
}

/// Render error envelope to stderr (never suppressed by quiet mode).
pub fn emit_error(
    envelope: &ErrorEnvelopeV1,
    cfg: EmitterConfig,
) -> Result<RenderedOutput, EmitError> {
    let value = serde_json::to_value(envelope)?;

    let mut content = match cfg.format {
        OutputFormat::Text => {
            let msg = envelope.error.message.as_str();
            colorize_error(msg, cfg)
        }
        _ => render_value(&value, cfg)?,
    };

    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok(RenderedOutput { stream: OutputStream::Stderr, content })
}

/// Format debug log line when debug/trace logging is enabled.
#[must_use]
pub fn format_debug_log(message: &str, cfg: EmitterConfig) -> Option<String> {
    match cfg.log_level {
        LogLevel::Trace | LogLevel::Debug => Some(format!("DEBUG {message}")),
        _ => None,
    }
}

/// Backward-compatible JSON rendering helper for marker types.
pub fn to_json(marker: &ContractMarker) -> Result<String, serde_json::Error> {
    serde_json::to_string(marker)
}

/// Backward-compatible YAML rendering helper for marker types.
pub fn to_yaml(marker: &ContractMarker) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(marker)
}
