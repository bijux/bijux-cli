#![forbid(unsafe_code)]
//! Schema snapshot tests to detect accidental drift.

use bijux_cli_contracts::schema::{
    error_envelope_v1_schema, output_envelope_v1_schema, plugin_manifest_v1_schema,
};
use proptest as _;
use semver as _;
use serde as _;

fn assert_snapshot(schema: schemars::schema::RootSchema, path: &str) {
    let rendered = serde_json::to_string_pretty(&schema).expect("schema should serialize");
    let expected = std::fs::read_to_string(path).expect("snapshot file should exist");
    assert_eq!(rendered, expected, "schema drift for {path}");
}

#[test]
fn output_schema_matches_snapshot() {
    assert_snapshot(output_envelope_v1_schema(), "tests/snapshots/output_envelope_v1.schema.json");
}

#[test]
fn error_schema_matches_snapshot() {
    assert_snapshot(error_envelope_v1_schema(), "tests/snapshots/error_envelope_v1.schema.json");
}

#[test]
fn plugin_schema_matches_snapshot() {
    assert_snapshot(plugin_manifest_v1_schema(), "tests/snapshots/plugin_manifest_v1.schema.json");
}
