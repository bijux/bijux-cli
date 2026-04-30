#![forbid(unsafe_code)]
//! Output encoding and envelope rendering surfaces for core app execution.

use crate::contracts::{ColorMode, ErrorEnvelopeV1, LogLevel, OutputEnvelopeV1, OutputFormat};
use serde_json::Value;
use std::io::IsTerminal;

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
            format: OutputFormat::Text,
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
        ColorMode::Auto => std::io::stdout().is_terminal() || std::io::stderr().is_terminal(),
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

fn render_jsonl(value: &Value) -> Result<String, EmitError> {
    match value {
        Value::Array(items) => {
            let mut lines = Vec::with_capacity(items.len());
            for item in items {
                lines.push(serde_json::to_string(item).map_err(EmitError::from)?);
            }
            Ok(lines.join("\n"))
        }
        _ => serde_json::to_string(value).map_err(EmitError::from),
    }
}

fn scalar_text(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_string()),
        Value::Bool(boolean) => Some(boolean.to_string()),
        Value::Number(number) => Some(number.to_string()),
        Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

fn render_text_lines(value: &Value, indent: usize, lines: &mut Vec<String>) {
    let pad = " ".repeat(indent);
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                lines.push(format!("{pad}{{}}"));
                return;
            }
            for (key, item) in map {
                if let Some(scalar) = scalar_text(item) {
                    lines.push(format!("{pad}{key}: {scalar}"));
                    continue;
                }
                if let Some(array) = item.as_array() {
                    if array.is_empty() {
                        lines.push(format!("{pad}{key}: []"));
                        continue;
                    }
                }
                if let Some(object) = item.as_object() {
                    if object.is_empty() {
                        lines.push(format!("{pad}{key}: {{}}"));
                        continue;
                    }
                }

                lines.push(format!("{pad}{key}:"));
                render_text_lines(item, indent + 2, lines);
            }
        }
        Value::Array(items) => {
            if items.is_empty() {
                lines.push(format!("{pad}[]"));
                return;
            }
            for item in items {
                if let Some(scalar) = scalar_text(item) {
                    lines.push(format!("{pad}- {scalar}"));
                    continue;
                }
                if let Some(array) = item.as_array() {
                    if array.is_empty() {
                        lines.push(format!("{pad}- []"));
                        continue;
                    }
                }
                if let Some(object) = item.as_object() {
                    if object.is_empty() {
                        lines.push(format!("{pad}- {{}}"));
                        continue;
                    }
                }

                lines.push(format!("{pad}-"));
                render_text_lines(item, indent + 2, lines);
            }
        }
        _ => lines.push(format!("{pad}{}", scalar_text(value).unwrap_or_default())),
    }
}

fn render_text(value: &Value) -> String {
    if let Some(scalar) = scalar_text(value) {
        return scalar;
    }

    let mut lines = Vec::new();
    render_text_lines(value, 0, &mut lines);
    lines.join("\n")
}

/// Render arbitrary value in configured format.
pub fn render_value(value: &Value, cfg: EmitterConfig) -> Result<String, EmitError> {
    match cfg.format {
        OutputFormat::Jsonl => render_jsonl(value),
        OutputFormat::Yaml => serde_yaml::to_string(value).map_err(EmitError::from),
        OutputFormat::Text => Ok(render_text(value)),
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
mod tests {
    use super::{render_value, EmitterConfig};
    use crate::contracts::OutputFormat;
    use serde_json::json;

    #[test]
    fn render_value_jsonl_emits_one_line_per_array_item() {
        let cfg = EmitterConfig { format: OutputFormat::Jsonl, ..EmitterConfig::default() };
        let rendered = render_value(&json!([{"a": 1}, {"b": 2}]), cfg).expect("jsonl");
        assert_eq!(rendered, "{\"a\":1}\n{\"b\":2}");
    }

    #[test]
    fn render_value_jsonl_emits_single_line_for_object() {
        let cfg = EmitterConfig { format: OutputFormat::Jsonl, ..EmitterConfig::default() };
        let rendered = render_value(&json!({"status": "ok"}), cfg).expect("jsonl");
        assert_eq!(rendered, "{\"status\":\"ok\"}");
    }
}
