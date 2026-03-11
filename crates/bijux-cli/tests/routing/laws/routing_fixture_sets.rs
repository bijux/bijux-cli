#![forbid(unsafe_code)]
//! Fixture sets for cli/dev/plugin/invalid routing cases.

use std::fs;

use bijux_cli::api::routing::registry::{RouteRegistry, RouteTarget};
use proptest as _;
use serde as _;
use serde::Deserialize;
use serde_json as _;

#[derive(Debug, Deserialize)]
struct SuggestCase {
    input: String,
    expected: String,
}
use clap as _;
use schemars as _;
use semver as _;
use thiserror as _;

fn read_lines(path: &str) -> Vec<Vec<String>> {
    fs::read_to_string(path)
        .expect("fixture should exist")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            line.split_whitespace()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn all_cli_subcommand_fixtures_resolve() {
    let registry = RouteRegistry::default();
    for path in read_lines("tests/data/fixtures/routing/cli_subcommands.txt") {
        let resolved = registry.resolve(&path).expect("cli fixture should resolve");
        assert!(matches!(resolved, RouteTarget::BuiltIn));
    }
}

#[test]
fn all_dev_cli_subcommand_fixtures_resolve() {
    let registry = RouteRegistry::default();
    for path in read_lines("tests/data/fixtures/routing/dev_cli_subcommands.txt") {
        let resolved = registry
            .resolve(&path)
            .expect("dev cli fixture should resolve");
        assert!(matches!(resolved, RouteTarget::BuiltIn));
    }
}

#[test]
fn plugin_namespace_fixture_commands_resolve_after_registration() {
    let mut registry = RouteRegistry::default();
    registry
        .register_plugin_namespace("community")
        .expect("plugin namespace should register");

    for path in read_lines("tests/data/fixtures/routing/plugin_namespace_commands.txt") {
        let resolved = registry
            .resolve(&path)
            .expect("plugin fixture should resolve");
        assert!(matches!(resolved, RouteTarget::Plugin(ns) if ns == "community"));
    }
}

#[test]
fn invalid_command_suggestions_match_fixtures() {
    let text = fs::read_to_string("tests/data/fixtures/routing/invalid_command_suggestions.json")
        .expect("fixture should exist");
    let cases: Vec<SuggestCase> = serde_json::from_str(&text).expect("fixture json should parse");

    let registry = RouteRegistry::default();
    for case in cases {
        let actual = registry
            .suggest_namespace(&case.input)
            .expect("suggestion should exist");
        assert_eq!(actual, case.expected);
    }
}
