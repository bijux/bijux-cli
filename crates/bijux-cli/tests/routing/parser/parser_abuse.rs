#![forbid(unsafe_code)]
//! Adversarial parser and routing hardening tests.

use bijux_cli::api::routing::catalog::dev_cli_subcommands;
use bijux_cli::api::routing::parser::parse_intent;
use bijux_cli::api::routing::registry::{RouteError, RouteRegistry, RouteTarget};
use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use serde_json as _;
use thiserror as _;

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

fn pick<'a>(values: &'a [&'a str], seed: &mut u64) -> &'a str {
    let idx = (lcg(seed) as usize) % values.len();
    values[idx]
}

#[test]
fn randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry() {
    let roots = ["status", "audit", "docs", "doctor", "version", "history", "memory"];
    let cli_sub = ["status", "paths", "self-test", "config", "plugins"];
    let dev_sub = dev_cli_subcommands();
    let plugin_sub = ["list", "inspect", "check", "install", "uninstall", "doctor"];
    let junk = ["--unknown", "--format", "???", "", "###", "--log-level", "noise", "--color"];

    let mut seed = 0xBAD5_EED_u64;
    for _ in 0..160_u16 {
        let mut argv = vec!["bijux".to_string()];
        match lcg(&mut seed) % 4 {
            0 => argv.push(pick(&roots, &mut seed).to_string()),
            1 => {
                argv.push("cli".to_string());
                argv.push(pick(&cli_sub, &mut seed).to_string());
                if argv[2] == "config" {
                    argv.push(
                        pick(&["get", "set", "unset", "clear", "load"], &mut seed).to_string(),
                    );
                }
                if argv[2] == "plugins" {
                    argv.push(pick(&plugin_sub, &mut seed).to_string());
                }
            }
            2 => {
                argv.push("dev".to_string());
                argv.push("cli".to_string());
                argv.push(pick(&dev_sub, &mut seed).to_string());
            }
            _ => {
                argv.push("plugins".to_string());
                argv.push(pick(&plugin_sub, &mut seed).to_string());
            }
        }
        for _ in 0..(lcg(&mut seed) % 4) {
            argv.push(pick(&junk, &mut seed).to_string());
        }

        let parsed =
            parse_intent(&argv).expect("parser should not panic on randomized malformed corpus");
        assert!(parsed.command_path.len() <= 6, "path exploded for argv={argv:?}");
    }
}

#[test]
fn parser_handles_absurd_token_and_flag_lengths_and_empty_elements() {
    let huge_token = "x".repeat(32_768);
    let huge_value = "v".repeat(65_536);
    let cases = vec![
        vec!["bijux".to_string(), huge_token.clone()],
        vec!["bijux".to_string(), "status".to_string(), huge_token],
        vec!["bijux".to_string(), "--config-path".to_string(), huge_value],
        vec!["bijux".to_string(), "".to_string(), "status".to_string()],
    ];
    for argv in cases {
        let parsed = parse_intent(&argv).expect("parser should return deterministic fallback");
        assert!(parsed.command_path.len() <= 4);
    }
}

#[test]
fn parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic() {
    let one = parse_intent(&[
        "bijux".into(),
        "--pretty".into(),
        "--no-pretty".into(),
        "--pretty".into(),
        "--format".into(),
        "json".into(),
        "--text".into(),
        "--json".into(),
        "cli".into(),
        "status".into(),
    ])
    .expect("parse");
    let two = parse_intent(&[
        "bijux".into(),
        "cli".into(),
        "status".into(),
        "--json".into(),
        "--format".into(),
        "text".into(),
        "--pretty".into(),
        "--no-pretty".into(),
        "--pretty".into(),
    ])
    .expect("parse");
    assert_eq!(one.normalized_path, vec!["cli", "status"]);
    assert_eq!(two.normalized_path, vec!["cli", "status"]);
    assert_eq!(one.global_flags.output_format, Some(bijux_cli::api::routing::OutputFormat::Json));
    assert!(matches!(
        two.global_flags.output_format,
        Some(
            bijux_cli::api::routing::OutputFormat::Json
                | bijux_cli::api::routing::OutputFormat::Text
        )
    ));
    assert_eq!(one.global_flags.pretty_mode, Some(bijux_cli::api::routing::PrettyMode::Pretty));
    assert_eq!(two.global_flags.pretty_mode, Some(bijux_cli::api::routing::PrettyMode::Pretty));
}

#[test]
fn parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths() {
    let hostile = parse_intent(&["bijux".into(), "plugins".into(), "inspect;rm -rf /".into()])
        .expect("parse");
    assert!(hostile.normalized_path.is_empty());

    let confusable_dev =
        parse_intent(&["bijux".into(), "dеv".into(), "cli".into(), "status".into()])
            .expect("parse");
    // Cyrillic 'е' must not resolve to reserved `dev`.
    assert_ne!(confusable_dev.normalized_path, vec!["dev", "cli", "status"]);

    let confusable_help =
        parse_intent(&["bijux".into(), "hеlp".into(), "status".into()]).expect("parse");
    assert_ne!(confusable_help.normalized_path, vec!["help", "status"]);
}

#[test]
fn unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("plugin");
    registry.register_plugin_namespace("commander").expect("plugin");

    let suggested = registry.suggest_namespace("commnad").expect("suggestion should exist");
    assert!(suggested == "community" || suggested == "commander");

    let reserved_suggestion = registry.suggest_namespace("hepl").expect("suggestion");
    assert_eq!(reserved_suggestion, "help");

    let err = registry.resolve(&["help".to_string(), "version".to_string()]).expect_err("unknown");
    assert!(matches!(err, RouteError::Unknown(_)));
}

#[test]
fn plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots() {
    let mut registry = RouteRegistry::default();
    for blocked in ["help", "version", "dev", "cli", "doctor"] {
        let err =
            registry.register_plugin_namespace(blocked).expect_err("blocked namespace should fail");
        assert!(matches!(err, RouteError::Reserved(_) | RouteError::Conflict(_)));
    }

    registry.register_plugin_namespace("community").expect("valid plugin namespace");
    registry.register_plugin_namespace("registry").expect("non-reserved namespace should register");

    let resolved = registry
        .resolve(&["community".to_string(), "run".to_string()])
        .expect("plugin route should resolve");
    assert!(matches!(resolved, RouteTarget::Plugin(ns) if ns == "community"));

    let dev_registry = registry
        .resolve(&["dev".to_string(), "cli".to_string(), "registry".to_string()])
        .expect("canonical dev cli registry route must remain builtin");
    assert!(matches!(dev_registry, RouteTarget::BuiltIn));
}

#[test]
fn route_tree_and_command_tree_are_deterministic_under_shuffled_plugin_registration() {
    let plugin_names = ["gamma", "alpha", "omega", "beta"];

    let mut left = RouteRegistry::default();
    for name in plugin_names {
        left.register_plugin_namespace(name).expect("left registration");
    }

    let mut right = RouteRegistry::default();
    for name in plugin_names.iter().rev() {
        right.register_plugin_namespace(name).expect("right registration");
    }

    assert_eq!(left.route_tree(), right.route_tree());
    assert_eq!(left.render_command_tree(), right.render_command_tree());
}

#[test]
fn command_tree_export_is_stable_across_repeated_calls() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("community").expect("plugin");

    let first = registry.render_command_tree();
    let second = registry.render_command_tree();
    let third = registry.render_command_tree();

    assert_eq!(first, second);
    assert_eq!(second, third);
}
