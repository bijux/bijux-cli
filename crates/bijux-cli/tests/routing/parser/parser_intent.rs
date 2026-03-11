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
        "dev".to_string(),
        "cli".to_string(),
        "routes".to_string(),
    ];
    let two = vec![
        "bijux".to_string(),
        "--no-pretty".to_string(),
        "--pretty".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "routes".to_string(),
    ];
    let left = parse_intent(&one).expect("parse should succeed");
    let right = parse_intent(&two).expect("parse should succeed");
    assert_eq!(left.command_path, right.command_path);
    assert_eq!(left.normalized_path, right.normalized_path);
    assert_eq!(
        left.global_flags.output_format,
        right.global_flags.output_format
    );
    assert_eq!(
        left.global_flags.pretty_mode,
        right.global_flags.pretty_mode
    );
}
