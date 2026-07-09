#![forbid(unsafe_code)]
//! REPL session resilience coverage for stable behavior contracts.

use libc as _;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use bijux_cli as _;
use bijux_cli::api::repl::{
    completion_candidates, configure_history, execute_repl_input, execute_repl_line, startup_repl,
    startup_repl_with_diagnostics, ReplEvent, ReplInput, ReplStream,
};
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn temp_path(name: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "bijux-repl-session-resilience-{name}-{}.{}",
        std::process::id(),
        ext
    ))
}

#[test]
fn repeated_malformed_plugin_and_config_failures_recover_to_success() {
    let (mut session, _) = startup_repl("default", None);

    for _ in 0..3 {
        let err = execute_repl_input(&mut session, ReplInput::Line(":not-a-meta".to_string()))
            .expect_err("malformed meta should fail");
        assert!(err.to_string().contains("invalid repl command"));
    }

    for _ in 0..3 {
        let frame = execute_repl_line(&mut session, "community missing-subcommand")
            .expect("plugin route should return frame")
            .expect("plugin failure frame");
        assert_eq!(frame.stream, ReplStream::Stderr);
    }

    for _ in 0..3 {
        let frame = execute_repl_line(&mut session, "config get")
            .expect("config usage failure should return frame")
            .expect("config usage frame");
        assert_eq!(frame.stream, ReplStream::Stderr);
    }

    let recovered = execute_repl_line(&mut session, "status")
        .expect("session should recover")
        .expect("status frame");
    assert_eq!(recovered.stream, ReplStream::Stdout);
    let payload: Value = serde_json::from_str(&recovered.content).expect("status json");
    let status = payload["status"].as_str().expect("status should be a string");
    assert!(matches!(status, "ok" | "warning" | "degraded"));
}

#[test]
fn startup_with_corrupted_history_registry_missing_paths_and_large_history_is_resilient() {
    let history = temp_path("broken-history", "json");
    fs::write(&history, "{not-json\0").expect("write corrupted history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(history.clone()), true, 128);
    load_history_like_user(&mut session);

    let (_session_with_diag, _startup, diagnostics) =
        startup_repl_with_diagnostics("default", None, &["community"]);
    assert_eq!(diagnostics.len(), 1);

    let huge = temp_path("huge-history", "json");
    let huge_lines: Vec<String> = (0..50_000).map(|i| format!("status {i}")).collect();
    fs::write(&huge, serde_json::to_string(&huge_lines).expect("serialize huge"))
        .expect("write huge history");

    let (mut huge_session, _) = startup_repl("default", None);
    configure_history(&mut huge_session, Some(huge.clone()), true, 2_000);
    let started = Instant::now();
    load_history_like_user(&mut huge_session);
    let first =
        execute_repl_line(&mut huge_session, "status").expect("first command").expect("frame");
    assert!(started.elapsed() < Duration::from_secs(2), "first-command latency budget exceeded");
    assert_eq!(first.stream, ReplStream::Stdout);

    let _ = fs::remove_file(history);
    let _ = fs::remove_file(huge);
}

#[test]
fn ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session() {
    let (mut session, _) = startup_repl("default", None);

    execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
        .expect("json mode");
    let json_frame =
        execute_repl_line(&mut session, "status").expect("json status").expect("frame");
    assert!(json_frame.content.trim_start().starts_with('{'));

    execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
        .expect("yaml mode");
    let yaml_frame =
        execute_repl_line(&mut session, "status").expect("yaml status").expect("frame");
    assert!(yaml_frame.content.contains("status:"));

    execute_repl_input(&mut session, ReplInput::Line(":set format text".to_string()))
        .expect("text mode");
    let text_frame =
        execute_repl_line(&mut session, "status").expect("text status").expect("frame");
    assert!(text_frame.content.contains("status:"));

    let no_color =
        execute_repl_line(&mut session, "help status --color never").expect("help").expect("frame");
    assert!(!no_color.content.contains("\u{1b}["));

    execute_repl_line(&mut session, "status").expect("prepare interrupt");
    let interrupted = execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt");
    assert!(matches!(interrupted, ReplEvent::Interrupted(_)));

    execute_repl_line(&mut session, "community inspect").expect("plugin cmd");
    let interrupted_plugin =
        execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt plugin");
    assert!(matches!(interrupted_plugin, ReplEvent::Interrupted(_)));

    execute_repl_input(&mut session, ReplInput::Line("status \\".to_string()))
        .expect("multiline open");
    let eof = execute_repl_input(&mut session, ReplInput::Eof).expect("eof");
    assert!(matches!(eof, ReplEvent::Exit(_)));
}

#[test]
fn plugin_management_doctor_and_broken_completion_source_do_not_crash() {
    let (mut session, _) = startup_repl("default", None);

    for command in ["plugins install", "plugins list", "plugins check"] {
        let frame = execute_repl_line(&mut session, command)
            .expect("plugin command should return frame")
            .expect("plugin frame");
        assert!(matches!(frame.stream, ReplStream::Stdout | ReplStream::Stderr));
    }

    let config = temp_path("broken-config", "env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN_LINE\n").expect("broken config");
    let doctor = execute_repl_line(
        &mut session,
        &format!("doctor --format json --no-pretty --config-path {}", config.display()),
    )
    .expect("doctor command")
    .expect("doctor frame");
    assert!(matches!(doctor.stream, ReplStream::Stdout | ReplStream::Stderr));

    let (_broken, _startup, diagnostics) =
        startup_repl_with_diagnostics("default", None, &["community", "missing"]);
    assert!(!diagnostics.is_empty());

    let suggestions = completion_candidates(&session, "sta");
    assert!(suggestions.iter().any(|c| c == "status"));

    let recovered =
        execute_repl_line(&mut session, "status").expect("recovery").expect("recovery frame");
    assert_eq!(recovered.stream, ReplStream::Stdout);

    let _ = fs::remove_file(config);
}

fn load_history_like_user(session: &mut bijux_cli::api::repl::ReplSession) {
    // Mirrors user startup behavior with tolerant history loading.
    let _ = bijux_cli::api::repl::load_history(session);
}
