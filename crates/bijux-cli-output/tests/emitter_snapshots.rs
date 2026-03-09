#![forbid(unsafe_code)]
//! Snapshot tests for emitter outputs.

use bijux_cli_contracts::{
    ColorMode, CommandPath, ErrorEnvelopeV1, ErrorPayloadV1, LogLevel, Namespace,
    OutputEnvelopeMetaV1, OutputEnvelopeV1, OutputFormat,
};
use bijux_cli_output::{emit_error, emit_success, EmitterConfig};
use serde_json::json;
use serde_yaml as _;
use bijux_cli_core as _;
use serde as _;
use thiserror as _;

fn sample_meta() -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: vec![Namespace("cli".to_string()), Namespace("status".to_string())],
        },
        timestamp: "1970-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn snapshots_for_success_emitters() {
    let envelope = OutputEnvelopeV1 {
        status: "ok".to_string(),
        data: json!({"healthy": true}),
        meta: sample_meta(),
    };

    let pretty_json = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: true, ..EmitterConfig::default() },
    )
    .expect("emit should succeed")
    .expect("output expected")
    .content;
    assert_eq!(pretty_json, include_str!("snapshots/success_json_pretty.txt"));

    let compact_json = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Json, pretty: false, ..EmitterConfig::default() },
    )
    .expect("emit should succeed")
    .expect("output expected")
    .content;
    assert_eq!(compact_json, include_str!("snapshots/success_json_compact.txt"));

    let yaml = emit_success(
        &envelope,
        EmitterConfig { format: OutputFormat::Yaml, pretty: true, ..EmitterConfig::default() },
    )
    .expect("emit should succeed")
    .expect("output expected")
    .content;
    assert_eq!(yaml, include_str!("snapshots/success_yaml.txt"));
}

#[test]
fn snapshots_for_error_categories() {
    for (category, code, message, snap) in [
        ("usage", "usage_error", "Usage failed", include_str!("snapshots/error_usage.json")),
        (
            "validation",
            "validation_error",
            "Validation failed",
            include_str!("snapshots/error_validation.json"),
        ),
        ("plugin", "plugin_error", "Plugin failed", include_str!("snapshots/error_plugin.json")),
        (
            "internal",
            "internal_error",
            "Internal failed",
            include_str!("snapshots/error_internal.json"),
        ),
    ] {
        let envelope = ErrorEnvelopeV1 {
            status: "error".to_string(),
            error: ErrorPayloadV1 {
                code: code.to_string(),
                message: message.to_string(),
                category: category.to_string(),
                details: None,
            },
            meta: sample_meta(),
        };

        let rendered = emit_error(
            &envelope,
            EmitterConfig {
                format: OutputFormat::Json,
                pretty: false,
                color: ColorMode::Never,
                log_level: LogLevel::Info,
                quiet: false,
                no_color: true,
            },
        )
        .expect("emit should succeed")
        .content;
        assert_eq!(rendered, snap);
    }
}
