#![forbid(unsafe_code)]
//! Root grammar contract tests.

use bijux_cli::api::routing::parser::{parse_intent, root_command};

#[test]
fn root_help_surfaces_iteration_one_foundation_commands() {
    let root_argv = vec!["bijux".to_string(), "--help".to_string()];
    let root_help = match root_command().try_get_matches_from(root_argv) {
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            error.to_string()
        }
        other => panic!("expected clap help output, got {other:?}"),
    };

    for token in ["explain", "apps", "plugins", "cli"] {
        assert!(root_help.contains(token), "root help missing `{token}`");
    }
    assert!(root_help.contains("jsonl"), "help must advertise jsonl output mode");

    let cli_argv = vec!["bijux".to_string(), "cli".to_string(), "--help".to_string()];
    let cli_help = match root_command().try_get_matches_from(cli_argv) {
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            error.to_string()
        }
        other => panic!("expected clap help output, got {other:?}"),
    };
    for token in ["script-contract", "routes", "shims", "doctor", "plugins"] {
        assert!(cli_help.contains(token), "cli help missing `{token}`");
    }
}

#[test]
fn alias_and_foundation_routes_normalize_deterministically() {
    let explain = parse_intent(&["bijux".to_string(), "explain".to_string(), "status".to_string()])
        .expect("parse explain");
    assert_eq!(explain.normalized_path, vec!["explain"]);

    let config_diff = parse_intent(&[
        "bijux".to_string(),
        "config".to_string(),
        "diff".to_string(),
        "--from-profile".to_string(),
        "dev".to_string(),
    ])
    .expect("parse config diff");
    assert_eq!(config_diff.normalized_path, vec!["cli", "config", "diff"]);

    let cli_routes = parse_intent(&["bijux".to_string(), "cli".to_string(), "routes".to_string()])
        .expect("parse cli routes");
    assert_eq!(cli_routes.normalized_path, vec!["cli", "routes"]);
}

#[test]
fn root_parser_accepts_all_contract_output_modes() {
    for mode in ["json", "jsonl", "yaml", "text"] {
        let intent = parse_intent(&[
            "bijux".to_string(),
            "--format".to_string(),
            mode.to_string(),
            "status".to_string(),
        ])
        .expect("parse format");
        assert!(intent.global_flags.output_format.is_some(), "mode {mode} must parse");
    }
}
