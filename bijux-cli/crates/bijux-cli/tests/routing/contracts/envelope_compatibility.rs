#![forbid(unsafe_code)]

//! Backward/forward compatibility tests for envelope contracts.

use std::collections::BTreeMap;

use bijux_cli::contracts::{
    CommandPath, ErrorEnvelopeV1, ErrorPayloadV1, Namespace, OutputEnvelopeMetaV1, OutputEnvelopeV1,
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
