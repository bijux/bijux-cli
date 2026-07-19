#![forbid(unsafe_code)]
//! REPL transcript contract coverage for runtime behavior.

use libc as _;
use std::fs;
use std::time::{Duration, Instant};

use bijux_cli as _;
use bijux_cli::api::repl::{
    completion_candidates, configure_history, execute_repl_input, execute_repl_line,
    inspect_last_error, load_history, register_completion_registry,
    register_plugin_completion_hook, repl_argv_from_line, startup_repl,
    startup_repl_with_diagnostics, ReplEvent, ReplInput,
};
use bijux_cli::api::routing::parser::parse_intent;
use bijux_cli::api::runtime::run_app;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_history_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("bijux-repl-transcript-{name}.txt"))
}

#[test]
fn repl_transcript_help_command() {
    let (mut session, _) = startup_repl("default", None);
    let event = execute_repl_input(&mut session, ReplInput::Line(":help status".to_string()))
        .expect("help should execute");
    match event {
        ReplEvent::Continue(Some(frame)) => assert!(frame.content.contains("Usage:")),
        _ => panic!("unexpected help event"),
    }
}

#[test]
fn repl_transcript_plugin_command() {
    let (mut session, _) = startup_repl("default", None);
    let frame = execute_repl_line(&mut session, "community inspect").expect("plugin route");
    let content = frame.expect("frame").content;
    assert!(!content.trim().is_empty());
}

#[test]
fn repl_transcript_error_command() {
    let (mut session, _) = startup_repl("default", None);
    let err = execute_repl_input(&mut session, ReplInput::Line(":invalid".to_string()))
        .expect_err("invalid meta command should fail");
    assert!(err.to_string().contains("invalid repl command"));
    assert!(inspect_last_error(&session).is_some());
}

#[test]
fn repl_transcript_quiet_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set quiet on".to_string()))
        .expect("quiet on");
    let frame = execute_repl_line(&mut session, "status").expect("status line");
    assert!(frame.is_none());
}

#[test]
fn repl_transcript_json_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
        .expect("json mode");
    let frame = execute_repl_line(&mut session, "status").expect("json line");
    assert!(frame.expect("frame").content.trim_start().starts_with('{'));
}

#[test]
fn repl_transcript_yaml_mode() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
        .expect("yaml mode");
    let frame = execute_repl_line(&mut session, "status").expect("yaml line");
    assert!(frame.expect("frame").content.contains("status:"));
}

#[test]
fn repl_transcript_interrupt() {
    let (mut session, _) = startup_repl("default", None);
    let interrupted = execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt");
    assert!(matches!(interrupted, ReplEvent::Interrupted(_)));
}

#[test]
fn repl_transcript_eof_exit() {
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
fn history_file_supports_cli_json_layout_for_repl_interop() {
    let path = temp_history_path("cli-json-layout");
    fs::write(
        &path,
        "[{\"command\":\"status\",\"timestamp\":1.0},{\"command\":\"plugins list\",\"timestamp\":2.0}]",
    )
    .expect("write history");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(path.clone()), true, 10);
    load_history(&mut session).expect("load history");

    assert_eq!(session.history, vec!["status", "plugins list"]);
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
fn completion_includes_runtime_registry_candidates() {
    let (mut session, _) = startup_repl("default", None);
    register_completion_registry(
        &mut session,
        "runtime-catalog",
        vec!["history".to_string(), "history clear".to_string(), "memory".to_string()],
    );
    let history = completion_candidates(&session, "hist");
    let cli = completion_candidates(&session, "cli");

    assert!(history.iter().any(|s| s == "history"));
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
    let lines =
        (0..20_000).map(|idx| format!("status --item {idx}")).collect::<Vec<_>>().join("\n");
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

#[test]
fn repl_transcript_contracts_cover_status_doctor_plugins_config_get_and_history() {
    let (mut session, _) = startup_repl("default", None);
    let commands =
        ["status", "doctor", "plugins list", "cli config get repl_missing_key", "history"];

    for command in commands {
        let frame = execute_repl_line(&mut session, command).expect("command should execute");
        assert!(frame.is_some(), "expected frame for command: {command}");
    }
}

#[test]
fn repl_transcript_command_failure_recovery_and_syntax_errors() {
    let (mut session, _) = startup_repl("default", None);
    let failed = execute_repl_line(&mut session, "inspect unexpected")
        .expect("execution should return frame");
    let failed_frame = failed.expect("failure should emit frame");
    assert_eq!(failed_frame.stream, bijux_cli::api::repl::ReplStream::Stderr);
    assert!(failed_frame.content.contains("Usage: bijux"));

    let recovered = execute_repl_line(&mut session, "status").expect("session should recover");
    assert!(recovered.expect("status frame").content.contains("\"status\""));
}

#[test]
fn repl_transcript_nested_help_inside_repl() {
    let (mut session, _) = startup_repl("default", None);
    let event = execute_repl_input(&mut session, ReplInput::Line(":help cli status".to_string()))
        .expect("nested help should execute");
    match event {
        ReplEvent::Continue(Some(frame)) => {
            assert!(frame.content.contains("Usage:"));
            assert!(frame.content.contains("status"));
        }
        _ => panic!("unexpected nested help event"),
    }
}

#[test]
fn repl_transcript_switching_output_formats_in_session() {
    let (mut session, _) = startup_repl("default", None);

    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
        .expect("json mode");
    let json_frame = execute_repl_line(&mut session, "status").expect("json status");
    assert!(json_frame.expect("json frame").content.trim_start().starts_with('{'));

    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
        .expect("yaml mode");
    let yaml_frame = execute_repl_line(&mut session, "status").expect("yaml status");
    assert!(yaml_frame.expect("yaml frame").content.contains("status:"));

    let _ = execute_repl_input(&mut session, ReplInput::Line(":set format text".to_string()))
        .expect("text mode");
    let text_frame = execute_repl_line(&mut session, "status").expect("text status");
    assert!(text_frame.expect("text frame").content.contains("status:"));
}

#[test]
fn repl_transcript_trace_mode_in_session() {
    let (mut session, _) = startup_repl("default", None);
    let _ = execute_repl_input(&mut session, ReplInput::Line(":set trace on".to_string()))
        .expect("trace mode on");
    assert!(session.trace_mode);

    let traced = execute_repl_line(&mut session, "status").expect("status line");
    assert!(traced.expect("frame").content.contains("\"status\""));

    let _ = execute_repl_input(&mut session, ReplInput::Line(":set trace off".to_string()))
        .expect("trace mode off");
    assert!(!session.trace_mode);
}

#[test]
fn completion_covers_grouped_cli_and_partial_tokens() {
    let (mut session, _) = startup_repl("default", None);
    register_completion_registry(
        &mut session,
        "runtime-catalog",
        vec!["history".to_string(), "history clear".to_string(), "memory".to_string()],
    );
    let cli = completion_candidates(&session, "cli");
    let history = completion_candidates(&session, "hist");
    let partial = completion_candidates(&session, "mem");

    assert!(cli.iter().any(|value| value.starts_with("cli")));
    assert!(history.iter().any(|value| value == "history"));
    assert!(partial.iter().any(|value| value == "memory"));
}

#[test]
fn reserved_namespace_collision_diagnostics_are_reported_in_session() {
    let (mut session, _) = startup_repl("default", None);
    let temp = std::env::temp_dir().join("bijux-repl-reserved-namespace");
    let frame = execute_repl_line(
        &mut session,
        &format!("cli plugins scaffold python cli --path {}", temp.display()),
    )
    .expect("command should execute");
    let failure = frame.expect("expected diagnostics frame");
    assert_eq!(failure.stream, bijux_cli::api::repl::ReplStream::Stderr);
    assert!(failure.content.contains("reserved"));
}

#[test]
fn repl_does_not_define_separate_semantics_for_common_commands() {
    let (mut session, _) = startup_repl("default", None);
    let commands = ["status", "doctor", "history"];

    for command in commands {
        let repl = execute_repl_line(&mut session, command).expect("repl command").expect("frame");
        let repl_json: serde_json::Value = serde_json::from_str(&repl.content).expect("repl json");

        let cli = run_app(&[
            "bijux".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--pretty".to_string(),
            "--color".to_string(),
            "never".to_string(),
            "--log-level".to_string(),
            "info".to_string(),
            command.to_string(),
        ])
        .expect("cli run");
        let cli_json: serde_json::Value = serde_json::from_str(&cli.stdout).expect("cli json");
        assert_eq!(repl_json, cli_json, "semantic mismatch for command: {command}");
    }
}

#[test]
fn repl_startup_latency_stays_within_loaded_registry_budget() {
    let start = Instant::now();
    let (_session, _startup, diagnostics) = startup_repl_with_diagnostics(
        "default",
        None,
        &["community", "memory", "history", "plugins", "doctor"],
    );
    let elapsed = start.elapsed();
    assert_eq!(diagnostics.len(), 5);
    assert!(elapsed < Duration::from_millis(200));
}

#[test]
fn repl_output_parity_with_non_interactive_cli_for_status() {
    let (mut session, _) = startup_repl("default", None);
    let repl = execute_repl_line(&mut session, "status").expect("repl status").expect("repl frame");
    assert_eq!(repl.stream, bijux_cli::api::repl::ReplStream::Stdout);
    let repl_value: serde_json::Value = serde_json::from_str(&repl.content).expect("repl json");

    let cli = run_app(&[
        "bijux".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--pretty".to_string(),
        "--color".to_string(),
        "never".to_string(),
        "--log-level".to_string(),
        "info".to_string(),
        "status".to_string(),
    ])
    .expect("cli run");
    let cli_value: serde_json::Value = serde_json::from_str(&cli.stdout).expect("cli json");

    assert_eq!(repl_value, cli_value);
}
