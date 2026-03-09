#![forbid(unsafe_code)]
//! Parser intent normalization tests.

use bijux_cli_contracts as _;
use bijux_cli_routing::parser::parse_intent;
use clap as _;
use proptest as _;
use strsim as _;
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
    assert_eq!(intent.normalized_path, vec!["cli", "status"]);
    assert!(intent.global_flags.quiet);
    assert!(intent.global_flags.output_format.is_some());
    assert!(intent.global_flags.log_level.is_some());
}

#[test]
fn normalizes_dev_cli_alias_forms() {
    let argv = vec!["bijux".to_string(), "dev".to_string(), "routes".to_string()];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.normalized_path, vec!["dev", "cli", "routes"]);
}
