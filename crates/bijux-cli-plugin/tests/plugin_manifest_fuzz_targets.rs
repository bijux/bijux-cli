#![forbid(unsafe_code)]
//! Plugin manifest fuzz targets for parsing and validation boundaries.
//! test_type: plugin-manifest-fuzz

use bijux_cli_contracts as _;
use bijux_cli_plugin::{parse_manifest_v1, validate_manifest, PluginError};
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

fn manifest_json(namespace: &str, kind: &str, entrypoint: &str, aliases: &str, compatibility: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {compatibility},
  "namespace": "{namespace}",
  "kind": "{kind}",
  "aliases": {aliases},
  "entrypoint": "{entrypoint}",
  "capabilities": [{{"name":"exec","version":"1"}}]
}}"#
    )
}

#[test]
fn fuzz_plugin_manifest_parsing_is_stable() {
    let corpus = [
        "{broken-json",
        "{}",
        &manifest_json("alpha", "delegated", "alpha.plugin:run", "[]", r#"{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}"#),
    ];

    for sample in corpus {
        let a = parse_manifest_v1(sample);
        let b = parse_manifest_v1(sample);
        assert_eq!(a.is_ok(), b.is_ok());
    }
}

#[test]
fn fuzz_plugin_manifest_validation_covers_required_and_optional_fields() {
    let good = parse_manifest_v1(&manifest_json(
        "alpha",
        "delegated",
        "alpha.plugin:run",
        "[\"alpha-a\"]",
        r#"{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}"#,
    ))
    .expect("parse");
    let validated = validate_manifest(good, "0.1.0", &[]).expect("validate");
    assert_eq!(validated.manifest.namespace.0, "alpha");

    let with_empty_optional_aliases = parse_manifest_v1(&manifest_json(
        "beta",
        "delegated",
        "beta.plugin:run",
        "[]",
        r#"{"min_inclusive":"0.1.0","max_exclusive":null}"#,
    ))
    .expect("parse");
    let validated_optional = validate_manifest(with_empty_optional_aliases, "0.1.0", &[]).expect("validate");
    assert!(validated_optional.manifest.aliases.is_empty());
}

#[test]
fn fuzz_compatibility_range_parsing_is_enforced() {
    let bad = parse_manifest_v1(&manifest_json(
        "compat",
        "delegated",
        "compat.plugin:run",
        "[]",
        r#"{"min_inclusive":"not-semver","max_exclusive":"2.0.0"}"#,
    ))
    .expect("parse bad");
    let err = validate_manifest(bad, "0.1.0", &[]).expect_err("invalid min version must fail");
    assert!(matches!(err, PluginError::InvalidField(_)));

    let host_outside = parse_manifest_v1(&manifest_json(
        "compat2",
        "delegated",
        "compat2.plugin:run",
        "[]",
        r#"{"min_inclusive":"9.0.0","max_exclusive":"10.0.0"}"#,
    ))
    .expect("parse outside");
    let err2 = validate_manifest(host_outside, "0.1.0", &[]).expect_err("incompatible must fail");
    assert!(matches!(err2, PluginError::IncompatibleVersion { .. }));
}

#[test]
fn fuzz_plugin_entrypoint_path_parsing_by_kind_is_enforced() {
    let delegated_bad = parse_manifest_v1(&manifest_json(
        "entrya",
        "delegated",
        "entrya",
        "[]",
        r#"{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}"#,
    ))
    .expect("parse");
    let delegated_err = validate_manifest(delegated_bad, "0.1.0", &[]).expect_err("delegated entrypoint invalid");
    assert!(matches!(delegated_err, PluginError::InvalidEntrypoint { .. }));

    let external_bad = parse_manifest_v1(&manifest_json(
        "entryb",
        "external-exec",
        "binary:main",
        "[]",
        r#"{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}"#,
    ))
    .expect("parse");
    let external_err = validate_manifest(external_bad, "0.1.0", &[]).expect_err("external entrypoint invalid");
    assert!(matches!(external_err, PluginError::InvalidEntrypoint { .. }));
}

#[test]
fn fuzz_plugin_metadata_optional_fields_and_duplicate_aliases() {
    let duplicate_alias = parse_manifest_v1(&manifest_json(
        "meta",
        "delegated",
        "meta.plugin:run",
        "[\"dupe\",\"DUPE\"]",
        r#"{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}"#,
    ))
    .expect("parse");
    let err = validate_manifest(duplicate_alias, "0.1.0", &[]).expect_err("duplicate aliases fail");
    assert!(matches!(err, PluginError::DuplicateAlias(_)));
}
