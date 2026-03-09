#![forbid(unsafe_code)]
//! Expanded transcript parity and resiliency cases for REPL runtime.

use std::fs;
use std::time::{Duration, Instant};

use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_output as _;
use bijux_cli_repl::{
    completion_candidates, configure_history, execute_repl_input, execute_repl_line,
    inspect_last_error, load_history, register_plugin_completion_hook, repl_argv_from_line,
    startup_repl, startup_repl_with_diagnostics, ReplEvent, ReplInput,
};
use bijux_cli_routing::parser::parse_intent;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_history_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("bijux-repl-case-{name}.txt"))
}

#[test]
fn transcript_case_help_command() {
    let (mut session, _) = startup_repl("default", None);
    let event = execute_repl_input(&mut session, ReplInput::Line(":help status".to_string()))
        .expect("help should execute");
    match event {
        ReplEvent::Continue(Some(frame)) => assert!(frame.content.contains("Usage:")),
        _ => panic!("unexpected help event"),
    }
}

#[test]
fn transcript_case_plugin_command() {
    let (mut session, _) = startup_repl("default", None);
    let frame = execute_repl_line(&mut session, "community inspect").expect("plugin route");
    let content = frame.expect("frame").content;
    assert!(!content.trim().is_empty());
}

#[test]
fn transcript_case_error_command() {
    let (mut session, _) = startup_repl("default", None);
    let err = execute_repl_input(&mut session, ReplInput::Line(":invalid".to_string()))
        .expect_err("invalid meta command should fail");
    assert!(err.to_string().contains("invalid repl command"));
    assert!(inspect_last_error(&session).is_some());
}

#[test]
fn transcript_case_quiet_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set quiet on".to_string()))
        .expect("quiet on");
    let frame = execute_repl_line(&mut session, "status").expect("status line");
    assert!(frame.is_none());
}

#[test]
fn transcript_case_json_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
        .expect("json mode");
    let frame = execute_repl_line(&mut session, "status").expect("json line");
    assert!(frame.expect("frame").content.trim_start().starts_with('{'));
}

#[test]
fn transcript_case_yaml_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
        .expect("yaml mode");
    let frame = execute_repl_line(&mut session, "status").expect("yaml line");
    assert!(frame.expect("frame").content.contains("status:"));
}

#[test]
fn transcript_case_interrupt() {
    let (mut session, _) = startup_repl("default", None);
    let interrupted = execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt");
    assert!(matches!(interrupted, ReplEvent::Interrupted(_)));
}

#[test]
fn transcript_case_eof_exit() {
    let (mut session, _) = startup_repl("default", None);
    let eof = execute_repl_input(&mut session, ReplInput::Eof).expect("eof");
    assert!(matches!(eof, ReplEvent::Exit(_)));
}

#[test]
fn history_file_supports_python_prompt_toolkit_layout() {
    let path = temp_history_path("python-layout");
    fs::write(&path, "status\ndoctor\ncommunity inspect\n").expect("write history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(path.clone()), true, 10);
    load_history(&mut session).expect("load history");

    assert_eq!(session.history, vec!["status", "doctor", "community inspect"]);
    let _ = fs::remove_file(path);
}

#[test]
fn repl_line_tokenization_matches_cli_parser_expectations() {
    let argv = repl_argv_from_line("status --format json --no-pretty");
    let parsed = parse_intent(&argv).expect("repl argv should parse");

    let expected = parse_intent(&[
        "bijux".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ])
    .expect("expected argv should parse");

    assert_eq!(parsed.normalized_path, expected.normalized_path);
    assert_eq!(parsed.global_flags.output_format, expected.global_flags.output_format);
}

#[test]
fn completion_includes_reserved_namespace_candidates() {
    let (session, _) = startup_repl("default", None);
    let atlas = completion_candidates(&session, "atl");
    let cli = completion_candidates(&session, "cli");

    assert!(atlas.iter().any(|s| s == "atlas"));
    assert!(cli.iter().any(|s| s == "cli"));
}

#[test]
fn completion_includes_plugin_namespace_candidates() {
    let (mut session, _) = startup_repl("default", None);
    register_plugin_completion_hook(
        &mut session,
        "community",
        vec!["community inspect".to_string(), "community status".to_string()],
    );
    let values = completion_candidates(&session, "community");
    assert!(values.iter().any(|s| s == "community"));
    assert!(values.iter().any(|s| s == "community inspect"));
}

#[test]
fn malformed_history_recovers_without_crashing() {
    let path = temp_history_path("malformed");
    fs::write(&path, "{not-json\u{0}").expect("write malformed history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(path.clone()), true, 50);
    load_history(&mut session).expect("load should tolerate corruption");

    assert!(session.history.is_empty());
    assert!(inspect_last_error(&session).is_some());
    let _ = fs::remove_file(path);
}

#[test]
fn large_history_load_stays_within_sanity_budget() {
    let path = temp_history_path("perf");
    let lines = (0..20_000)
        .map(|idx| format!("status --item {idx}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, format!("{lines}\n")).expect("write history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(path.clone()), true, 20_000);

    let started = Instant::now();
    load_history(&mut session).expect("load history");
    let elapsed = started.elapsed();

    assert_eq!(session.history.len(), 20_000);
    assert!(elapsed < Duration::from_secs(2));
    let _ = fs::remove_file(path);
}

#[test]
fn startup_works_without_config_or_plugin_registry() {
    let (_session, _startup) = startup_repl("default", None);
    let (_session2, _startup2, diagnostics) =
        startup_repl_with_diagnostics("default", None, &["community"]);
    assert_eq!(diagnostics.len(), 1);
}
