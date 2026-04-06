#![forbid(unsafe_code)]
//! REPL completion coverage for stable behavior contracts.

use libc as _;
use std::fs;

use bijux_cli as _;
use bijux_cli::api::repl::{
    completion_candidates, configure_history, load_history, register_completion_registry,
    register_plugin_completion_hook, startup_repl, startup_repl_with_diagnostics,
};
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_history_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("bijux-repl-completion-{name}.txt"))
}

#[test]
fn completion_empty_prompt_and_partial_root_cli_tokens_are_supported() {
    let (mut session, _) = startup_repl("default", None);
    register_completion_registry(
        &mut session,
        "runtime-catalog",
        vec!["history".to_string(), "history clear".to_string(), "memory".to_string()],
    );

    let root = completion_candidates(&session, "");
    let root_partial = completion_candidates(&session, "sta");
    let cli_partial = completion_candidates(&session, "cli st");
    let history_partial = completion_candidates(&session, "hist");

    assert!(root.iter().any(|c| c == "status"));
    assert!(root.iter().any(|c| c == "cli"));
    assert!(root.iter().any(|c| c == "history"));
    assert!(root_partial.iter().any(|c| c == "status"));
    assert!(cli_partial.iter().any(|c| c == "cli status"));
    assert!(history_partial.iter().any(|c| c == "history"));
}

#[test]
fn completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported() {
    let (mut session, _) = startup_repl("default", None);
    register_plugin_completion_hook(
        &mut session,
        "community",
        vec![
            "community inspect".to_string(),
            "community lint".to_string(),
            "config get".to_string(),
            "plugins list".to_string(),
            "doctor".to_string(),
        ],
    );

    let plugin_ns = completion_candidates(&session, "comm");
    let config = completion_candidates(&session, "config g");
    let plugin = completion_candidates(&session, "plugins l");
    let diagnostics = completion_candidates(&session, "doct");

    assert!(plugin_ns.iter().any(|c| c == "community"));
    assert!(config.iter().any(|c| c == "config get"));
    assert!(plugin.iter().any(|c| c == "plugins list"));
    assert!(diagnostics.iter().any(|c| c == "doctor"));
}

#[test]
fn completion_runtime_namespaces_are_visible_and_aliases_are_not_rewritten() {
    let (mut session, _) = startup_repl("default", None);
    register_completion_registry(
        &mut session,
        "runtime-catalog",
        vec!["history".to_string(), "history clear".to_string(), "memory".to_string()],
    );
    let reserved = completion_candidates(&session, "cli");
    let history = completion_candidates(&session, "hist");

    assert!(reserved.iter().any(|c| c == "cli"));
    assert!(reserved.iter().any(|c| c == "cli status"));
    assert!(history.iter().any(|c| c == "history"));
    assert!(!history.iter().any(|c| c == "history status"));
}

#[test]
fn completion_recovers_with_broken_registry_corrupted_state_and_no_plugins() {
    let (mut session, _, diagnostics) =
        startup_repl_with_diagnostics("default", None, &["community", "memory"]);
    assert_eq!(diagnostics.len(), 2);

    let path = temp_history_path("corrupted");
    fs::write(&path, "{not-json\u{0}").expect("write malformed history");
    configure_history(&mut session, Some(path.clone()), true, 50);
    load_history(&mut session).expect("load should tolerate corrupted history");

    let no_plugins = completion_candidates(&session, "community");
    let under_corruption = completion_candidates(&session, "sta");
    assert!(no_plugins.is_empty());
    assert!(under_corruption.iter().any(|c| c == "status"));

    let _ = fs::remove_file(path);
}

#[test]
fn completion_ordering_is_stable_with_multiple_plugins_and_repeated_runs() {
    let (mut session, _) = startup_repl("default", None);
    register_plugin_completion_hook(
        &mut session,
        "zeta",
        vec!["zeta doctor".to_string(), "zeta status".to_string()],
    );
    register_plugin_completion_hook(
        &mut session,
        "alpha",
        vec!["alpha check".to_string(), "alpha inspect".to_string()],
    );

    let once = completion_candidates(&session, "");
    let twice = completion_candidates(&session, "");

    let mut expected = once.clone();
    expected.sort();
    expected.dedup();

    assert_eq!(once, expected);
    assert_eq!(once, twice);
    assert!(once.iter().any(|c| c == "alpha"));
    assert!(once.iter().any(|c| c == "zeta"));
}

#[test]
fn completion_includes_full_config_history_and_memory_command_sets() {
    let (session, _) = startup_repl("default", None);
    let all = completion_candidates(&session, "");

    for required in [
        "config list",
        "config get",
        "config set",
        "config unset",
        "config clear",
        "config reload",
        "config export",
        "config load",
        "history clear",
        "memory list",
        "memory get",
        "memory set",
        "memory delete",
        "memory clear",
    ] {
        assert!(
            all.iter().any(|candidate| candidate == required),
            "missing completion candidate: {required}"
        );
    }
}
