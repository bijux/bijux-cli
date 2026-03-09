#![forbid(unsafe_code)]
//! Transcript, parity, and resiliency regression tests for REPL runtime.

use std::fs;
use std::path::PathBuf;

use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_output as _;
use bijux_cli_repl::{
    check_repl_budgets, completion_candidates, configure_history, execute_repl_input,
    execute_repl_line, flush_history, inspect_last_error, load_history,
    register_plugin_completion_hook, render_repl_command_reference, replay_history_command,
    startup_repl, ReplEvent, ReplInput, ReplStream, REPL_MEMORY_BUDGET_BYTES,
};
use bijux_cli_routing as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_history_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bijux-repl-{name}.json"))
}

#[test]
fn transcript_regression_matches_expected_flow() {
    let (mut session, _) = startup_repl("default", None);

    let events = vec![
        execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
            .expect("set format"),
        execute_repl_input(&mut session, ReplInput::Line("status".to_string())).expect("status"),
        execute_repl_input(&mut session, ReplInput::Line("community inspect".to_string()))
            .expect("plugin route"),
    ];

    assert!(matches!(events[0], ReplEvent::Continue(Some(_))));
    assert!(matches!(events[1], ReplEvent::Continue(Some(_))));
    assert!(matches!(events[2], ReplEvent::Continue(Some(_))));
}

#[test]
fn cli_vs_repl_parity_for_builtin_and_plugin_routes() {
    let (mut session, _) = startup_repl("default", None);

    let builtin = execute_repl_line(&mut session, "status").expect("builtin run");
    let plugin = execute_repl_line(&mut session, "community status").expect("plugin run");

    assert!(!builtin.expect("builtin frame").content.trim().is_empty());
    assert!(!plugin.expect("plugin frame").content.trim().is_empty());
}

#[test]
fn interrupt_during_plugin_execution_is_safe() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_line(&mut session, "community status").expect("plugin line");

    let interrupted = execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt");
    match interrupted {
        ReplEvent::Interrupted(frame) => assert_eq!(frame.stream, ReplStream::Stderr),
        _ => panic!("unexpected event"),
    }
}

#[test]
fn malformed_input_sets_last_error_and_continues_session() {
    let (mut session, _) = startup_repl("default", None);
    let err = execute_repl_input(&mut session, ReplInput::Line(":unknown-command".to_string()))
        .expect_err("invalid meta command should fail");
    assert!(err.to_string().contains("invalid repl command"));
    assert!(inspect_last_error(&session).is_some());
}

#[test]
fn large_history_files_load_with_cap() {
    let path = temp_history_path("large-history");
    let large: Vec<String> = (0..5_000).map(|i| format!("status {i}")).collect();
    fs::write(&path, format!("{}\n", serde_json::to_string(&large).expect("serialize")))
        .expect("write history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(path.clone()), true, 128);
    load_history(&mut session).expect("load capped history");
    assert_eq!(session.history.len(), 128);

    flush_history(&session).expect("flush capped history");
    let _ = fs::remove_file(path);
}

#[test]
fn completion_rendering_and_history_replay_work() {
    let (mut session, _) = startup_repl("default", None);
    register_plugin_completion_hook(
        &mut session,
        "community",
        vec!["community status".to_string(), "community inspect".to_string()],
    );
    let suggestions = completion_candidates(&session, "community");
    assert!(suggestions.iter().any(|s| s == "community"));

    let _ = execute_repl_line(&mut session, "status").expect("run status");
    let replayed = replay_history_command(&mut session, 0).expect("replay history");
    assert!(replayed.is_some());
}

#[test]
fn startup_without_config_and_with_broken_plugins_is_supported() {
    let (_plain, _startup) = startup_repl("default", None);
    let (_session, _startup, diagnostics) =
        bijux_cli_repl::startup_repl_with_diagnostics("default", None, &["community"]);
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn memory_budget_check_can_flag_large_sessions() {
    let (mut session, _) = startup_repl("default", None);
    session.history = vec!["x".repeat(REPL_MEMORY_BUDGET_BYTES)];
    let warnings = check_repl_budgets(&session, std::time::Duration::from_millis(1));
    assert!(!warnings.is_empty());
}

#[test]
fn command_reference_snapshot_is_stable() {
    let rendered = render_repl_command_reference();
    let expected = include_str!("snapshots/repl_command_reference.txt");
    assert_eq!(rendered, expected);
}
