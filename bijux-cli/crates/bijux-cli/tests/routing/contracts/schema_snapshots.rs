#![forbid(unsafe_code)]

//! Schema snapshot tests to detect accidental drift.

use bijux_cli::contracts::{
    error_envelope_v1_schema, output_envelope_v1_schema, plugin_manifest_v2_schema,
};
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use thiserror as _;

fn render(schema: &schemars::schema::RootSchema) -> String {
    serde_json::to_string_pretty(schema).expect("schema should serialize")
}

#[test]
fn schema_snapshots_are_deterministic_and_match_expected_files() {
    let cases = [
        (
            output_envelope_v1_schema as fn() -> schemars::schema::RootSchema,
            "tests/routing/snapshots/output_envelope_v1.schema.json",
        ),
        (
            error_envelope_v1_schema as fn() -> schemars::schema::RootSchema,
            "tests/routing/snapshots/error_envelope_v1.schema.json",
        ),
        (
            plugin_manifest_v2_schema as fn() -> schemars::schema::RootSchema,
            "tests/routing/snapshots/plugin_manifest_v2.schema.json",
        ),
    ];

    for (builder, path) in cases {
        let first_schema = builder();
        let second_schema = builder();
        let first = render(&first_schema);
        let second = render(&second_schema);
        assert_eq!(first, second, "schema generation must be repeated-run deterministic: {path}");

        let expected = std::fs::read_to_string(path).expect("snapshot file should exist");
        assert_eq!(first, expected, "schema drift for {path}");
    }
}
