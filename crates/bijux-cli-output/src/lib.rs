#![forbid(unsafe_code)]
//! Output encoding and envelope rendering surfaces.

use bijux_cli_routing::{ColorMode, ErrorEnvelopeV1, LogLevel, OutputEnvelopeV1, OutputFormat};
use serde_json::Value;

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
        ColorMode::Never => false,
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

fn with_trailing_newline(mut content: String) -> String {
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

fn render_json(value: &Value, pretty: bool) -> Result<String, EmitError> {
    if pretty {
        serde_json::to_string_pretty(value).map_err(EmitError::from)
    } else {
        serde_json::to_string(value).map_err(EmitError::from)
    }
}

/// Render arbitrary value in configured format.
pub fn render_value(value: &Value, cfg: EmitterConfig) -> Result<String, EmitError> {
    match cfg.format {
        OutputFormat::Yaml => serde_yaml::to_string(value).map_err(EmitError::from),
        OutputFormat::Text => {
            if let Some(text) = value.as_str() {
                Ok(text.to_string())
            } else if cfg.pretty {
                render_json(value, true)
            } else {
                render_json(value, false)
            }
        }
        _ => render_json(value, cfg.pretty),
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
    let content = with_trailing_newline(render_value(&value, cfg)?);

    Ok(Some(RenderedOutput { stream: OutputStream::Stdout, content }))
}

/// Render error envelope to stderr (never suppressed by quiet mode).
pub fn emit_error(
    envelope: &ErrorEnvelopeV1,
    cfg: EmitterConfig,
) -> Result<RenderedOutput, EmitError> {
    let value = serde_json::to_value(envelope)?;

    let content = match cfg.format {
        OutputFormat::Text => {
            let msg = envelope.error.message.as_str();
            colorize_error(msg, cfg)
        }
        _ => with_trailing_newline(render_value(&value, cfg)?),
    };
    Ok(RenderedOutput { stream: OutputStream::Stderr, content: with_trailing_newline(content) })
}

#[cfg(test)]
use bijux_cli_core as _;
#[cfg(test)]
use serde as _;
