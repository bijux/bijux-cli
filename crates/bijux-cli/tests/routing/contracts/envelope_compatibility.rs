#![forbid(unsafe_code)]

//! Backward/forward compatibility tests for envelope contracts.

use std::collections::BTreeMap;

use bijux_cli::contracts::{
    CommandEnvelopeV1, CommandErrorSummaryV1, CommandPath, CommandWarningV1, ErrorEnvelopeV1,
    ErrorPayloadV1, Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1,
};
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use serde_json::json;
use thiserror as _;

fn meta() -> OutputEnvelopeMetaV1 {
    OutputEnvelopeMetaV1 {
        version: "v1".to_string(),
        command: CommandPath {
            segments: vec![Namespace("cli".to_string()), Namespace("status".to_string())],
        },
        timestamp: "2026-03-09T00:00:00Z".to_string(),
    }
}

#[test]
fn older_error_details_without_context_deserializes() {
    let payload = json!({
        "status": "error",
        "error": {
            "code": "invalid_format",
            "message": "Unsupported format",
            "category": "usage",
            "details": { "failure": "invalid_format" }
        },
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z"
        }
    });

    let parsed: ErrorEnvelopeV1 = serde_json::from_value(payload).expect("compatible envelope");
    let details = parsed.error.details.expect("details should exist");
    assert_eq!(details.failure.as_deref(), Some("invalid_format"));
    assert_eq!(details.context, BTreeMap::new());
}

#[test]
fn unknown_optional_fields_are_ignored() {
    let payload = json!({
        "status": "error",
        "error": {
            "code": "invalid_format",
            "message": "Unsupported format",
            "category": "usage",
            "details": {
                "failure": "invalid_format",
                "context": {"flag": "--format"},
                "future_note": "ignored"
            },
            "future_error_field": true
        },
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z",
            "future_meta_field": "ignored"
        }
    });

    let parsed: ErrorEnvelopeV1 =
        serde_json::from_value(payload).expect("unknown fields should be tolerated");
    assert_eq!(parsed.error.code, "invalid_format");
}

#[test]
fn missing_optional_fields_are_accepted() {
    let payload = json!({
        "status": "error",
        "error": {
            "code": "validation_failed",
            "message": "bad input",
            "category": "validation"
        },
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z"
        }
    });

    let parsed: ErrorEnvelopeV1 = serde_json::from_value(payload).expect("details are optional");
    assert!(parsed.error.details.is_none());
}

#[test]
fn invalid_required_fields_are_rejected() {
    let missing_error = json!({
        "status": "error",
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z"
        }
    });
    assert!(serde_json::from_value::<ErrorEnvelopeV1>(missing_error).is_err());

    let wrong_status_type = json!({
        "status": {"not": "a-string"},
        "error": {
            "code": "invalid_format",
            "message": "Unsupported format",
            "category": "usage"
        },
        "meta": {
            "version": "v1",
            "command": {"segments": ["cli", "status"]},
            "timestamp": "2026-03-09T00:00:00Z"
        }
    });
    assert!(serde_json::from_value::<ErrorEnvelopeV1>(wrong_status_type).is_err());
}

#[test]
fn output_envelope_constructor_builds_success_status() {
    let envelope = OutputEnvelopeV1::success(json!({"status": "ok"}), meta());
    assert_eq!(envelope.status, "ok");
}

#[test]
fn error_payload_constructor_enforces_required_fields() {
    assert!(ErrorPayloadV1::new("", "msg", "usage").is_err());
    assert!(ErrorPayloadV1::new("code", "", "usage").is_err());
    assert!(ErrorPayloadV1::new("code", "msg", "").is_err());
    assert!(ErrorPayloadV1::new("code", "msg", "usage").is_ok());
}

#[test]
fn command_envelope_constructor_enforces_success_and_error_invariants() {
    let command = CommandPath::new(&["cli", "status"]).expect("command path");
    let warning = CommandWarningV1::new("degraded_path", "degraded runtime path")
        .expect("warning");
    let error = CommandErrorSummaryV1::new("usage.missing_arg", "Missing argument: KEY")
        .expect("error");

    let ok = CommandEnvelopeV1::new(
        "command-envelope-v1",
        command.clone(),
        true,
        "ok",
        json!({"status":"ok"}),
        vec![warning],
        Vec::new(),
        "2026-04-30T00:00:00Z",
    );
    assert!(ok.is_ok());

    let invalid_success_with_error = CommandEnvelopeV1::new(
        "command-envelope-v1",
        command.clone(),
        true,
        "ok",
        json!({}),
        Vec::new(),
        vec![error.clone()],
        "2026-04-30T00:00:00Z",
    );
    assert!(invalid_success_with_error.is_err());

    let invalid_failure_without_error = CommandEnvelopeV1::new(
        "command-envelope-v1",
        command,
        false,
        "usage.missing_arg",
        json!({}),
        Vec::new(),
        Vec::new(),
        "2026-04-30T00:00:00Z",
    );
    assert!(invalid_failure_without_error.is_err());

    let failed = CommandEnvelopeV1::new(
        "command-envelope-v1",
        CommandPath::new(&["cli", "config", "get"]).expect("command"),
        false,
        "usage.missing_arg",
        json!({}),
        Vec::new(),
        vec![error],
        "2026-04-30T00:00:00Z",
    );
    assert!(failed.is_ok());
}

#[test]
fn command_envelope_fixtures_round_trip_as_stable_contract() {
    for fixture in [
        "tests/data/fixtures/routing/command_envelope_success_v1.json",
        "tests/data/fixtures/routing/command_envelope_failure_v1.json",
    ] {
        let raw = std::fs::read_to_string(fixture).expect("read fixture");
        let parsed: CommandEnvelopeV1 = serde_json::from_str(&raw).expect("fixture contract");
        let rendered = serde_json::to_string_pretty(&parsed).expect("render fixture");
        let expected: serde_json::Value = serde_json::from_str(&raw).expect("fixture json");
        let actual: serde_json::Value = serde_json::from_str(&rendered).expect("roundtrip json");
        assert_eq!(actual, expected, "fixture drift for {fixture}");
    }
}
