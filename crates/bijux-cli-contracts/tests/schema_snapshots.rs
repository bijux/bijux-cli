#![forbid(unsafe_code)]
//! Schema snapshot tests to detect accidental drift.

use bijux_cli_contracts::schema::{
    error_envelope_v1_schema, output_envelope_v1_schema, plugin_manifest_v1_schema,
};
use proptest as _;
use semver as _;
use serde as _;

fn render(schema: schemars::schema::RootSchema) -> String {
    serde_json::to_string_pretty(&schema).expect("schema should serialize")
}

#[test]
fn schema_snapshots_are_deterministic_and_match_expected_files() {
    let cases = [
        (
            output_envelope_v1_schema as fn() -> schemars::schema::RootSchema,
            "tests/snapshots/output_envelope_v1.schema.json",
        ),
        (
            error_envelope_v1_schema as fn() -> schemars::schema::RootSchema,
            "tests/snapshots/error_envelope_v1.schema.json",
        ),
        (
            plugin_manifest_v1_schema as fn() -> schemars::schema::RootSchema,
            "tests/snapshots/plugin_manifest_v1.schema.json",
        ),
    ];

    for (builder, path) in cases {
        let first = render(builder());
        let second = render(builder());
        assert_eq!(first, second, "schema generation must be repeated-run deterministic: {path}");

        let expected = std::fs::read_to_string(path).expect("snapshot file should exist");
        assert_eq!(first, expected, "schema drift for {path}");
    }
}
