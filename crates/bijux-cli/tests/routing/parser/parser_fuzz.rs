#![forbid(unsafe_code)]
//! Fuzz-style parser robustness checks.
//! test_type: flag-precedence-conflict

use bijux_cli::routing::parser::parse_intent;
use proptest as _;
use serde as _;
use serde_json as _;

use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

#[test]
fn parser_returns_usage_intent_for_diverse_argv_without_panics() {
    let corpus = [
        vec!["bijux"],
        vec!["bijux", "status"],
        vec!["bijux", "cli", "status"],
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "--quiet", "plugins", "inspect"],
        vec!["bijux", "plugins", "list", "--format", "yaml"],
        vec![
            "bijux",
            "--color",
            "always",
            "--log-level",
            "trace",
            "doctor",
        ],
        vec!["bijux", "config", "get", "--no-pretty"],
        vec!["bijux", "--unknown"],
        vec!["bijux", "dev", "cli", "contracts", "--format", "json"],
    ];

    for case in corpus {
        let argv: Vec<String> = case.iter().map(|s| (*s).to_string()).collect();
        let intent = parse_intent(&argv).expect("parser should return usage intent, not crash");
        assert!(!intent.normalized_path.is_empty() || intent.command_path.is_empty());
    }
}
