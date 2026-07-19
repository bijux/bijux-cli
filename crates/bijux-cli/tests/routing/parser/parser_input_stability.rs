#![forbid(unsafe_code)]
//! Parser input stability checks for argv normalization and route safety.
//! test_type: parser-input-stability

use bijux_cli::api::routing::catalog::normalize_command_path;
use bijux_cli::api::routing::parser::{parse_intent, ParsedIntent};
use bijux_cli::api::routing::registry::{RouteError, RouteRegistry};
use bijux_cli::contracts::official_product_namespaces;
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
    values[(lcg(seed) as usize) % values.len()]
}

fn random_suffix(seed: &mut u64, max_items: usize) -> Vec<String> {
    let junk = [
        "--unknown",
        "--format",
        "json",
        "yaml",
        "--color",
        "never",
        "--log-level",
        "trace",
        "--pretty",
        "--no-pretty",
        "--quiet",
        "",
        "###",
    ];
    let mut out = Vec::new();
    for _ in 0..(lcg(seed) as usize % max_items.max(1)) {
        out.push(pick(&junk, seed).to_string());
    }
    out
}

fn stable_parse(argv: &[String]) -> Result<ParsedIntent, String> {
    parse_intent(argv).map_err(|err| format!("{err:?}"))
}

#[test]
fn fuzz_root_argv_parsing_does_not_panic() {
    let roots = ["status", "audit", "docs", "doctor", "version", "history", "memory", "help"];
    let mut seed = 0xA11C_E001_u64;

    for _ in 0..220 {
        let mut argv = vec!["bijux".to_string(), pick(&roots, &mut seed).to_string()];
        argv.extend(random_suffix(&mut seed, 4));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 6);
        }
    }
}

#[test]
fn fuzz_cli_argv_parsing_does_not_panic() {
    let cli_sub =
        ["status", "paths", "self-test", "config", "plugins", "doctor", "inspect", "version"];
    let config_sub = ["get", "set", "unset", "clear", "list", "export", "load", "reload"];
    let plugin_sub =
        ["list", "inspect", "install", "uninstall", "doctor", "check", "where", "schema"];
    let mut seed = 0xA11C_E002_u64;

    for _ in 0..220 {
        let mut argv =
            vec!["bijux".to_string(), "cli".to_string(), pick(&cli_sub, &mut seed).to_string()];
        if argv[2] == "config" {
            argv.push(pick(&config_sub, &mut seed).to_string());
        }
        if argv[2] == "plugins" {
            argv.push(pick(&plugin_sub, &mut seed).to_string());
        }
        argv.extend(random_suffix(&mut seed, 5));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 8);
        }
    }
}

#[test]
fn fuzz_external_runtime_mount_argv_parsing_does_not_panic() {
    let product_sub = ["status", "doctor", "config", "custom-command"];
    let mut seed = 0xA11C_E003_u64;

    for _ in 0..240 {
        let mut argv = vec![
            "bijux".to_string(),
            pick(&["atlas", "rag", "vex"], &mut seed).to_string(),
            pick(&product_sub, &mut seed).to_string(),
        ];
        argv.extend(random_suffix(&mut seed, 5));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 8);
        }
    }
}

#[test]
fn fuzz_plugin_command_argv_parsing_does_not_panic() {
    let plugin_sub = [
        "list",
        "inspect",
        "install",
        "uninstall",
        "doctor",
        "check",
        "reserved-names",
        "schema",
        "where",
        "explain",
    ];
    let mut seed = 0xA11C_E004_u64;

    for _ in 0..220 {
        let mut argv = vec![
            "bijux".to_string(),
            "plugins".to_string(),
            pick(&plugin_sub, &mut seed).to_string(),
        ];
        argv.extend(random_suffix(&mut seed, 6));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 7);
        }
    }
}

#[test]
fn fuzz_config_command_argv_parsing_does_not_panic() {
    let sub = ["get", "set", "unset", "clear", "list", "export", "load", "reload"];
    let mut seed = 0xA11C_E005_u64;

    for _ in 0..220 {
        let mut argv =
            vec!["bijux".to_string(), "config".to_string(), pick(&sub, &mut seed).to_string()];
        argv.extend(random_suffix(&mut seed, 6));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 7);
        }
    }
}

#[test]
fn fuzz_diagnostics_command_argv_parsing_does_not_panic() {
    let mut seed = 0xA11C_E006_u64;
    let commands = [
        vec!["bijux", "doctor"],
        vec!["bijux", "inspect"],
        vec!["bijux", "atlas", "doctor"],
        vec!["bijux", "rag", "status"],
    ];

    for _ in 0..220 {
        let mut argv = commands[(lcg(&mut seed) as usize) % commands.len()]
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
        argv.extend(random_suffix(&mut seed, 6));
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for argv={argv:?}");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 8);
        }
    }
}

#[test]
fn fuzz_mixed_global_local_flag_ordering_is_deterministic() {
    let variants = [
        vec!["bijux", "--format", "json", "cli", "status", "--no-pretty", "--color", "never"],
        vec!["bijux", "cli", "status", "--format", "json", "--color", "never", "--no-pretty"],
        vec!["bijux", "--color", "never", "cli", "status", "--no-pretty", "--format", "json"],
        vec!["bijux", "cli", "status", "--color", "never", "--format", "json", "--no-pretty"],
    ];

    let baseline = parse_intent(&variants[0].iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
        .expect("baseline");
    for variant in variants.iter().skip(1) {
        let parsed = parse_intent(&variant.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
            .expect("variant");
        assert_eq!(parsed.normalized_path, baseline.normalized_path);
        assert_eq!(parsed.global_flags.output_format, baseline.global_flags.output_format);
        assert_eq!(parsed.global_flags.color_mode, baseline.global_flags.color_mode);
    }
}

#[test]
fn fuzz_repeated_conflicting_flags_stays_safe_and_deterministic() {
    let mut seed = 0xA11C_E007_u64;
    let flags = ["--pretty", "--no-pretty", "--json", "--text", "--yaml", "--quiet", "--no-pretty"];

    for _ in 0..220 {
        let mut argv = vec!["bijux".to_string(), "cli".to_string(), "status".to_string()];
        for _ in 0..(2 + (lcg(&mut seed) as usize % 6)) {
            argv.push(pick(&flags, &mut seed).to_string());
        }
        let a = parse_intent(&argv).expect("first parse");
        let b = parse_intent(&argv).expect("second parse");
        assert_eq!(a.normalized_path, b.normalized_path);
        assert_eq!(a.global_flags, b.global_flags);
    }
}

#[test]
fn fuzz_huge_tokens_and_values_does_not_panic() {
    let huge_token = "x".repeat(65_536);
    let huge_value = "y".repeat(131_072);
    for argv in [
        vec!["bijux".to_string(), huge_token.clone()],
        vec!["bijux".to_string(), "status".to_string(), huge_token],
        vec!["bijux".to_string(), "--config-path".to_string(), huge_value],
    ] {
        let first = stable_parse(&argv);
        let second = stable_parse(&argv);
        assert_eq!(first.as_ref().err(), second.as_ref().err(), "error drift for huge argv");
        if let Ok(parsed) = first {
            assert!(parsed.command_path.len() <= 4);
        }
    }
}

#[test]
fn fuzz_typo_suggestion_paths_are_stable() {
    let registry = RouteRegistry::default();
    let mut seed = 0xA11C_E008_u64;
    let typos = ["hepl", "sttaus", "plguins", "inspec", "doktor", "versoin"];

    for _ in 0..180 {
        let typo = pick(&typos, &mut seed);
        let a = registry.suggest_namespace(typo);
        let b = registry.suggest_namespace(typo);
        assert_eq!(a, b, "suggestion drift for typo={typo}");
    }
}

#[test]
fn fuzz_help_path_parsing_and_alias_resolution_is_safe() {
    let cases = [
        vec!["bijux", "--help"],
        vec!["bijux", "help"],
        vec!["bijux", "cli", "--help"],
        vec!["bijux", "atlas", "--help"],
        vec!["bijux", "plugins", "inspect", "--help"],
        vec!["bijux", "rag", "status", "--help"],
    ];

    for case in cases {
        let parsed = parse_intent(&case.iter().map(|s| (*s).to_string()).collect::<Vec<_>>())
            .expect("help/alias parse should not fail");
        assert!(parsed.command_path.len() <= 6);
    }

    let alias =
        parse_intent(&["bijux".into(), "plugins".into(), "inspect".into()]).expect("alias parse");
    assert_eq!(alias.normalized_path, vec!["cli", "plugins", "inspect"]);

    let external_route =
        parse_intent(&["bijux".into(), "atlas".into(), "doctor".into()]).expect("mount parse");
    assert_eq!(external_route.normalized_path, vec!["atlas"]);
}

#[test]
fn fuzz_namespace_normalization_and_reserved_rejection_stays_safe() {
    let mut registry = RouteRegistry::default();
    registry.register_plugin_namespace("my-plugin").expect("baseline namespace");

    let normalized_collision =
        registry.register_plugin_namespace("my_plugin").expect_err("normalized collision");
    assert!(matches!(normalized_collision, RouteError::Conflict(_)));

    let case_collision =
        registry.register_plugin_namespace("MY-PLUGIN").expect_err("case collision");
    assert!(matches!(case_collision, RouteError::Conflict(_)));

    let mut reserved = RouteRegistry::default();
    for ns in official_product_namespaces() {
        let err =
            reserved.register_plugin_namespace(ns).expect_err("reserved namespace must reject");
        assert!(matches!(err, RouteError::Reserved(_) | RouteError::Conflict(_)));
    }

    for built_in in ["cli", "help", "version", "doctor", "plugins", "inspect"] {
        let err = reserved
            .register_plugin_namespace(built_in)
            .expect_err("built-in namespace must reject");
        assert!(matches!(err, RouteError::Reserved(_) | RouteError::Conflict(_)));
    }
}

#[test]
fn fuzz_reserved_name_rejection_and_normalization_are_deterministic() {
    let mut seed = 0xA11C_E009_u64;
    let candidates = ["cli", "help", "version", "doctor", "plugins", "status", "config"];

    for _ in 0..160 {
        let mut registry = RouteRegistry::default();
        let name = pick(&candidates, &mut seed);
        let a = registry.register_plugin_namespace(name);
        let mut registry_again = RouteRegistry::default();
        let b = registry_again.register_plugin_namespace(name);
        assert_eq!(a.is_ok(), b.is_ok(), "reserved-name rejection drift for {name}");

        let normalized = normalize_command_path(&["plugins".to_string(), "inspect".to_string()]);
        assert_eq!(normalized, vec!["cli", "plugins", "inspect"]);
    }
}
