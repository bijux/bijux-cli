#![forbid(unsafe_code)]

//! Property tests for namespace and command-path normalization.

use bijux_cli::contracts::{
    CommandPath, CompatibilityRange, Namespace, PluginCapability, PluginKind, PluginManifestV1,
};
use proptest::prelude::*;
use serde as _;
use serde_json as _;

proptest! {
    #[test]
    fn namespace_normalization_is_kebab_case(raw in "[A-Za-z0-9_ /-]{1,32}") {
        let normalized = Namespace::normalize(&raw);
        prop_assert!(normalized.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'));
        if normalized.is_empty() {
            prop_assert!(Namespace::new(&raw).is_err());
        } else {
            prop_assert!(!normalized.starts_with('-'));
            prop_assert!(!normalized.ends_with('-'));
            prop_assert!(!normalized.contains("--"));
            let ns = Namespace::new(&raw).expect("normalized namespace should be valid");
            prop_assert_eq!(ns.as_str(), normalized);
        }
    }

    #[test]
    fn command_path_roundtrip(raw_segments in proptest::collection::vec("[A-Za-z0-9][A-Za-z0-9_ -]{0,11}", 1..6)) {
        let refs: Vec<&str> = raw_segments.iter().map(String::as_str).collect();
        let path = CommandPath::new(&refs).expect("normalized path should be valid");
        let rendered = path.to_command_string();
        let reparsed = CommandPath::parse(&rendered).expect("rendered path should parse");
        prop_assert_eq!(path, reparsed);
    }
}

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn compatibility_range_supports_host_versions() {
    let range = CompatibilityRange::new("1.2.0", Some("2.0.0")).expect("valid range");
    assert!(range.supports_host("1.2.0").expect("valid host"));
    assert!(range.supports_host("1.9.9").expect("valid host"));
    assert!(!range.supports_host("2.0.0").expect("valid host"));
}

#[test]
fn plugin_manifest_constructor_enforces_invariants() {
    let compatibility = CompatibilityRange::new("1.0.0", Some("2.0.0")).expect("range");
    let namespace = Namespace::new("sample_plugin").expect("namespace");
    let capability = PluginCapability::new("inspect", Some("1")).expect("capability");

    let manifest = PluginManifestV1::new(
        "sample",
        "1.0.0",
        "1",
        "1",
        compatibility,
        namespace,
        PluginKind::Delegated,
        vec!["sample-status".to_string()],
        "sample_plugin:main",
        vec![capability],
    );

    assert!(manifest.is_ok());
}
