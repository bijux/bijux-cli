#![forbid(unsafe_code)]
//! Parser intent normalization tests.
//! `test_type`: flag-precedence-conflict

use bijux_cli::api::routing::parser::parse_intent;
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn parses_root_and_nested_paths_with_global_flags() {
    let argv = vec![
        "bijux".to_string(),
        "--quiet".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--log-level".to_string(),
        "debug".to_string(),
    ];

    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.command_path, vec!["status"]);
    assert_eq!(intent.normalized_path, vec!["status"]);
    assert!(intent.global_flags.quiet);
    assert!(intent.global_flags.output_format.is_some());
    assert!(intent.global_flags.log_level.is_some());
}

#[test]
fn conflicting_output_and_pretty_flags_normalize_deterministically() {
    let one = vec![
        "bijux".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
        "--pretty".to_string(),
        "--no-pretty".to_string(),
        "atlas".to_string(),
        "status".to_string(),
    ];
    let two = vec![
        "bijux".to_string(),
        "--no-pretty".to_string(),
        "--pretty".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "atlas".to_string(),
        "status".to_string(),
    ];
    let left = parse_intent(&one).expect("parse should succeed");
    let right = parse_intent(&two).expect("parse should succeed");
    assert_eq!(left.command_path, right.command_path);
    assert_eq!(left.normalized_path, right.normalized_path);
    assert_eq!(left.global_flags.output_format, right.global_flags.output_format);
    assert_eq!(left.global_flags.pretty_mode, right.global_flags.pretty_mode);
}

#[test]
fn completion_alias_normalizes_with_explicit_shell_selector() {
    let argv = vec![
        "bijux".to_string(),
        "completion".to_string(),
        "--shell".to_string(),
        "bash".to_string(),
    ];

    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.command_path, vec!["completion"]);
    assert_eq!(intent.normalized_path, vec!["cli", "completion"]);
}

#[test]
fn canonical_cli_completion_and_doctor_paths_parse_successfully() {
    let completion = parse_intent(&[
        "bijux".to_string(),
        "cli".to_string(),
        "completion".to_string(),
        "--shell".to_string(),
        "fish".to_string(),
    ])
    .expect("parse should succeed");
    assert_eq!(completion.command_path, vec!["cli", "completion"]);
    assert_eq!(completion.normalized_path, vec!["cli", "completion"]);

    let doctor = parse_intent(&["bijux".to_string(), "cli".to_string(), "doctor".to_string()])
        .expect("parse should succeed");
    assert_eq!(doctor.command_path, vec!["cli", "doctor"]);
    assert_eq!(doctor.normalized_path, vec!["cli", "doctor"]);
}

#[test]
fn config_docs_and_doctor_bundle_parse_successfully() {
    let config_docs = parse_intent(&[
        "bijux".to_string(),
        "config".to_string(),
        "docs".to_string(),
        "cli".to_string(),
    ])
    .expect("parse should succeed");
    assert_eq!(config_docs.command_path, vec!["config", "docs"]);
    assert_eq!(config_docs.normalized_path, vec!["cli", "config", "docs"]);

    let doctor_bundle = parse_intent(&[
        "bijux".to_string(),
        "doctor".to_string(),
        "--bundle".to_string(),
    ])
    .expect("parse should succeed");
    assert_eq!(doctor_bundle.command_path, vec!["doctor"]);
    assert_eq!(doctor_bundle.normalized_path, vec!["cli", "doctor"]);
}
