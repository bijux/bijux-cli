#![forbid(unsafe_code)]
//! Fuzz-style parser robustness checks.

use bijux_cli_contracts as _;
use bijux_cli_routing::parser::parse_intent;
use clap as _;
use proptest as _;
use serde as _;
use serde_json as _;
use thiserror as _;

#[test]
fn parser_handles_diverse_argv_inputs_without_panics() {
    let corpus = [
        vec!["bijux"],
        vec!["bijux", "status"],
        vec!["bijux", "cli", "status"],
        vec!["bijux", "dev", "cli", "routes"],
        vec!["bijux", "--quiet", "plugins", "inspect"],
        vec!["bijux", "plugins", "list", "--format", "yaml"],
        vec!["bijux", "--color", "always", "--log-level", "trace", "doctor"],
        vec!["bijux", "config", "get", "--no-pretty"],
        vec!["bijux", "--unknown"],
        vec!["bijux", "dev", "cli", "contracts", "--format", "json"],
    ];

    for case in corpus {
        let argv: Vec<String> = case.iter().map(|s| (*s).to_string()).collect();
        let result = parse_intent(&argv);
        assert!(result.is_ok(), "parser returned error for case: {case:?}");
    }
}
