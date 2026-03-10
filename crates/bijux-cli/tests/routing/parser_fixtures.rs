#![forbid(unsafe_code)]
//! Fixture-driven parser and route-resolution regression coverage.

use std::fs;

use bijux_cli::routing::parser::parse_intent;
use bijux_cli::routing::registry::{RouteRegistry, RouteTarget};
use proptest as _;
use serde as _;
use serde::Deserialize;
use serde_json as _;

#[derive(Debug, Deserialize)]
struct ParseCase {
    argv: Vec<String>,
    expected_normalized: Vec<String>,
    routed: bool,
}
use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

fn load_cases() -> Vec<ParseCase> {
    let text = fs::read_to_string("tests/routing/fixtures/parse_cases.json")
        .expect("fixture file should load");
    serde_json::from_str(&text).expect("fixture json should parse")
}

#[test]
fn parse_cases_match_expected_normalization_and_routing() {
    let cases = load_cases();
    let registry = RouteRegistry::default();

    for case in cases {
        let intent = parse_intent(&case.argv).expect("fixture argv should parse");
        assert_eq!(
            intent.normalized_path, case.expected_normalized,
            "normalized path mismatch for argv: {:?}",
            case.argv
        );

        let resolved = registry.resolve(&intent.normalized_path);
        match (case.routed, resolved) {
            (true, Ok(RouteTarget::BuiltIn | RouteTarget::Plugin(_))) => {}
            (true, other) => panic!("expected routed case, got: {other:?} for {:?}", case.argv),
            (false, Err(_)) => {}
            (false, other) => panic!("expected unrouted case, got: {other:?} for {:?}", case.argv),
        }
    }
}

#[test]
fn parser_repeated_flags_degrade_to_usage_intent_shape() {
    let argv = vec![
        "bijux".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert!(intent.normalized_path.is_empty());
    assert_eq!(intent.global_flags.output_format, None);
}

#[test]
fn parser_conflicting_pretty_flags_are_normalized_deterministically() {
    let argv = vec![
        "bijux".to_string(),
        "--pretty".to_string(),
        "--no-pretty".to_string(),
        "cli".to_string(),
        "status".to_string(),
    ];
    let intent = parse_intent(&argv).expect("parse should succeed");
    assert_eq!(intent.global_flags.pretty_mode, Some(bijux_cli::routing::PrettyMode::Compact));
}

#[test]
fn parser_flag_order_permutations_keep_same_result() {
    let variants = [
        vec!["bijux", "--quiet", "--format", "json", "cli", "status"],
        vec!["bijux", "cli", "status", "--quiet", "--format", "json"],
        vec!["bijux", "--format", "json", "cli", "status", "--quiet"],
    ];

    let baseline = parse_intent(&variants[0].iter().map(|x| x.to_string()).collect::<Vec<_>>())
        .expect("baseline parse");

    for variant in variants.iter().skip(1) {
        let intent = parse_intent(&variant.iter().map(|x| x.to_string()).collect::<Vec<_>>())
            .expect("variant parse");
        assert_eq!(intent.normalized_path, baseline.normalized_path);
        assert_eq!(intent.global_flags, baseline.global_flags);
    }
}

#[test]
fn parser_supports_global_flags_before_and_after_namespace() {
    let before = parse_intent(&[
        "bijux".to_string(),
        "--color".to_string(),
        "never".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "routes".to_string(),
    ])
    .expect("before parse");

    let after = parse_intent(&[
        "bijux".to_string(),
        "dev".to_string(),
        "cli".to_string(),
        "routes".to_string(),
        "--color".to_string(),
        "never".to_string(),
    ])
    .expect("after parse");

    assert_eq!(before.normalized_path, after.normalized_path);
    assert_eq!(before.global_flags, after.global_flags);
}

#[test]
fn empty_grouped_commands_are_detected() {
    let cli = parse_intent(&["bijux".to_string(), "cli".to_string()]).expect("parse cli");
    assert_eq!(cli.normalized_path, vec!["cli"]);

    let dev_cli = parse_intent(&["bijux".to_string(), "dev".to_string(), "cli".to_string()])
        .expect("parse dev cli");
    assert_eq!(dev_cli.normalized_path, vec!["dev", "cli"]);
}

#[test]
fn help_attached_at_multiple_levels_returns_help_intent_shape() {
    for argv in [
        vec!["bijux", "--help"],
        vec!["bijux", "cli", "--help"],
        vec!["bijux", "dev", "cli", "--help"],
    ] {
        let intent = parse_intent(&argv.iter().map(|x| x.to_string()).collect::<Vec<_>>())
            .expect("help parse should not error");
        assert!(intent.normalized_path.is_empty() || !intent.command_path.is_empty());
    }
}

#[test]
fn hidden_compatibility_aliases_are_normalized() {
    let cases = [
        (vec!["bijux", "status"], vec!["status"]),
        (vec!["bijux", "plugins", "inspect"], vec!["cli", "plugins", "inspect"]),
        (vec!["bijux", "dev", "doctor"], vec!["dev", "cli", "doctor"]),
    ];

    for (argv, expected) in cases {
        let intent = parse_intent(&argv.iter().map(|x| x.to_string()).collect::<Vec<_>>())
            .expect("parse should succeed");
        assert_eq!(intent.normalized_path, expected);
    }
}
