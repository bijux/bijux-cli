#![forbid(unsafe_code)]
//! Rendering edge-case and stream contract regressions.

use std::collections::BTreeMap;

use bijux_cli_core as _;
use bijux_cli_output::{emit_error, emit_success, render_value, EmitterConfig, OutputStream};
use bijux_cli_routing::{
    ColorMode, CommandPath, ErrorDetailsV1, ErrorEnvelopeV1, ErrorPayloadV1, LogLevel, Namespace,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
};
use serde as _;
use serde_json::json;
use serde_yaml as _;
use thiserror as _;

fn meta() -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: vec![Namespace("cli".to_string()), Namespace("status".to_string())],
        },
        timestamp: "1970-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn text_help_style_snapshots_for_root_and_grouped_help() {
    let root_help = json!("Usage: bijux-rs [OPTIONS] [COMMAND]\n\nCommands:\n  cli\n  dev\n");
    let grouped_help =
        json!("Usage: bijux-rs cli [OPTIONS] [COMMAND]\n\nCommands:\n  status\n  paths\n");

    let root_rendered = render_value(
        &root_help,
        EmitterConfig { format: OutputFormat::Text, ..EmitterConfig::default() },
    )
    .expect("root help should render");
    let grouped_rendered = render_value(
        &grouped_help,
        EmitterConfig { format: OutputFormat::Text, ..EmitterConfig::default() },
    )
    .expect("grouped help should render");

    assert_eq!(root_rendered, include_str!("snapshots/help_root_text.txt"));
    assert_eq!(grouped_rendered, include_str!("snapshots/help_grouped_text.txt"));
}

#[test]
fn stable_field_order_for_machine_output() {
    let envelope = OutputEnvelopeV1::success(json!({"alpha": 1, "beta": 2}), meta());

    let out = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit success")
    .expect("output present");

    assert_eq!(out.stream, OutputStream::Stdout);
    let out_repeat = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit success repeated")
    .expect("output present repeated");
    assert_eq!(out.content, out_repeat.content, "field ordering must be stable across renders");
}

#[test]
fn nested_error_details_and_multiline_unicode_rendering() {
    let err = ErrorEnvelopeV1::failure(
        ErrorPayloadV1 {
            code: "validation_failed".to_string(),
            message: "line1\nline2 – message".to_string(),
            category: "validation".to_string(),
            details: Some(ErrorDetailsV1 {
                failure: Some("invalid_key".to_string()),
                context: BTreeMap::from([
                    ("path".to_string(), json!("config.key")),
                    ("nested".to_string(), json!({"depth": [1,2,3]})),
                ]),
            }),
        },
        meta(),
    );

    let json_err = emit_error(
        &err,
        EmitterConfig {
            format: OutputFormat::Json,
            log_level: LogLevel::Debug,
            color: ColorMode::Never,
            ..EmitterConfig::default()
        },
    )
    .expect("json error emit");
    assert_eq!(json_err.stream, OutputStream::Stderr);
    assert!(json_err.content.contains("line1\\nline2"));
    assert!(json_err.content.contains("\"nested\""));

    let text_err = emit_error(
        &err,
        EmitterConfig {
            format: OutputFormat::Text,
            color: ColorMode::Always,
            no_color: false,
            ..EmitterConfig::default()
        },
    )
    .expect("text error emit");
    assert!(text_err.content.contains("\u{001b}[31m"));
}

#[test]
fn empty_payload_rendering_is_valid_in_all_formats() {
    let payload = json!({});

    let json_out = render_value(
        &payload,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("json render");
    assert_eq!(json_out, "{}");

    let yaml_out = render_value(
        &payload,
        EmitterConfig { format: OutputFormat::Yaml, pretty: true, ..EmitterConfig::default() },
    )
    .expect("yaml render");
    assert!(yaml_out.trim() == "{}" || yaml_out.trim().is_empty());

    let text_out = render_value(
        &payload,
        EmitterConfig { format: OutputFormat::Text, pretty: false, ..EmitterConfig::default() },
    )
    .expect("text render");
    assert_eq!(text_out, "{}");
}

#[test]
fn ansi_color_only_in_text_mode() {
    let err = ErrorEnvelopeV1::failure(
        ErrorPayloadV1::new("invalid", "problem", "usage").expect("valid payload"),
        meta(),
    );

    let json_err = emit_error(
        &err,
        EmitterConfig {
            format: OutputFormat::Json,
            color: ColorMode::Always,
            no_color: false,
            ..EmitterConfig::default()
        },
    )
    .expect("json error");
    assert!(!json_err.content.contains("\u{001b}[31m"));

    let text_err = emit_error(
        &err,
        EmitterConfig {
            format: OutputFormat::Text,
            color: ColorMode::Always,
            no_color: false,
            ..EmitterConfig::default()
        },
    )
    .expect("text error");
    assert!(text_err.content.contains("\u{001b}[31m"));
}
