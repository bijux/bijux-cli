#![forbid(unsafe_code)]
//! REPL hostile-session hardening coverage.

use libc as _;
use std::fs;
use std::path::PathBuf;

use bijux_cli::api::repl::{
    completion_candidates, configure_history, execute_repl_input, execute_repl_line,
    inspect_last_error, load_history, register_plugin_completion_hook, startup_repl,
    startup_repl_with_diagnostics, ReplEvent, ReplInput, ReplStream,
};
use bijux_cli::api::runtime::run_app;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn temp_path(name: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!("bijux-repl-hostile-{}-{}.{}", name, std::process::id(), ext))
}

#[test]
fn extremely_long_input_and_repeated_malformed_commands_recover() {
    let (mut session, _) = startup_repl("default", None);

    let long = format!("status {}", "x".repeat(256 * 1024));
    let err = execute_repl_line(&mut session, &long).expect_err("long line should be rejected");
    assert!(err.to_string().contains("command length limit exceeded"));
    assert_eq!(session.last_exit_code, 2);

    for _ in 0..5 {
        let err = execute_repl_input(&mut session, ReplInput::Line(":invalid-meta".to_string()))
            .expect_err("malformed command should error");
        assert!(err.to_string().contains("invalid repl command"));
    }
    assert!(inspect_last_error(&session).is_some());

    let recovered = execute_repl_line(&mut session, "status").expect("session should recover");
    let content = recovered.expect("status frame").content;
    assert!(content.contains("\"status\""));
}

#[test]
fn plugin_failure_config_readback_and_output_mode_switching_work_in_one_session() {
    let (mut session, _) = startup_repl("default", None);

    let failed_plugin = execute_repl_line(&mut session, "community definitely-missing-subcommand")
        .expect("plugin route should execute");
    let failed_plugin = failed_plugin.expect("plugin failure frame");
    assert_eq!(failed_plugin.stream, ReplStream::Stderr);

    let config_path = temp_path("config", "env");
    let _ = fs::remove_file(&config_path);
    let get_cmd = format!("cli config get alpha --config-path {}", config_path.display());

    let seeded = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "alpha=1".to_string(),
        "--config-path".to_string(),
        config_path.display().to_string(),
    ])
    .expect("seed config");
    assert_eq!(seeded.exit_code, 0);

    let _ = execute_repl_line(&mut session, &get_cmd).expect("config get");
    let core_get = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "alpha".to_string(),
        "--config-path".to_string(),
        config_path.display().to_string(),
    ])
    .expect("core get after repl set");
    assert_eq!(core_get.exit_code, 0);

    execute_repl_input(&mut session, ReplInput::Line(":set format json".to_string()))
        .expect("set json");
    let json = execute_repl_line(&mut session, "status").expect("status json").expect("frame");
    assert!(json.content.trim_start().starts_with('{'));

    execute_repl_input(&mut session, ReplInput::Line(":set format yaml".to_string()))
        .expect("set yaml");
    let yaml = execute_repl_line(&mut session, "status").expect("status yaml").expect("frame");
    assert!(yaml.content.contains("status:"));

    execute_repl_input(&mut session, ReplInput::Line(":set format text".to_string()))
        .expect("set text");
    let text = execute_repl_line(&mut session, "status").expect("status text").expect("frame");
    assert!(text.content.contains("status:"));

    let _ = fs::remove_file(config_path);
}

#[test]
fn quiet_trace_interrupt_and_eof_edge_cases_are_stable() {
    let (mut session, _) = startup_repl("default", None);

    execute_repl_input(&mut session, ReplInput::Line(":set quiet on".to_string()))
        .expect("quiet on");
    let quiet = execute_repl_line(&mut session, "status").expect("quiet status");
    assert!(quiet.is_none());

    execute_repl_input(&mut session, ReplInput::Line(":set quiet off".to_string()))
        .expect("quiet off");
    let loud = execute_repl_line(&mut session, "status").expect("loud status");
    assert!(loud.is_some());

    execute_repl_input(&mut session, ReplInput::Line(":set trace on".to_string()))
        .expect("trace on");
    assert!(session.trace_mode);
    execute_repl_line(&mut session, "status").expect("trace status");

    execute_repl_line(&mut session, "community inspect").expect("plugin command");
    let interrupt_plugin =
        execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt plugin");
    assert!(matches!(interrupt_plugin, ReplEvent::Interrupted(_)));

    execute_repl_line(&mut session, "status").expect("prepare config interrupt case");
    let interrupt_config =
        execute_repl_input(&mut session, ReplInput::Interrupt).expect("interrupt config");
    assert!(matches!(interrupt_config, ReplEvent::Interrupted(_)));

    execute_repl_input(&mut session, ReplInput::Line("status \\".to_string()))
        .expect("multiline pending");
    let eof = execute_repl_input(&mut session, ReplInput::Eof).expect("eof");
    assert!(matches!(eof, ReplEvent::Exit(None)));
    assert!(session.pending_multiline.is_none());
}

#[test]
fn completion_and_startup_recover_under_broken_registry_and_corrupted_state() {
    let (_session, _startup, diagnostics) =
        startup_repl_with_diagnostics("default", None, &["community"]);
    assert_eq!(diagnostics.len(), 1);

    let (mut session, _) = startup_repl("default", None);
    register_plugin_completion_hook(
        &mut session,
        "community",
        vec!["community status".to_string(), "community inspect".to_string()],
    );
    let plugin_completion = completion_candidates(&session, "community");
    assert!(plugin_completion.iter().any(|item| item == "community"));

    let history = temp_path("broken-history", "json");
    fs::write(&history, "{not-json\0").expect("write broken history");
    let (mut corrupted, _) = startup_repl("default", None);
    configure_history(&mut corrupted, Some(history.clone()), true, 64);
    load_history(&mut corrupted).expect("corrupted history should not crash");
    let after_corruption = completion_candidates(&corrupted, "sta");
    assert!(after_corruption.iter().any(|item| item == "status"));

    let huge_history = temp_path("huge-history", "json");
    let huge: Vec<String> = (0..25_000).map(|i| format!("status {i}")).collect();
    fs::write(&huge_history, serde_json::to_string(&huge).expect("serialize huge history"))
        .expect("write huge history");
    let (mut huge_session, _) = startup_repl("default", None);
    configure_history(&mut huge_session, Some(huge_history.clone()), true, 1_000);
    load_history(&mut huge_session).expect("huge history load");
    assert_eq!(huge_session.history.len(), 1_000);

    let _ = fs::remove_file(history);
    let _ = fs::remove_file(huge_history);
}

#[test]
fn repl_and_core_obey_same_command_result_law_for_shared_commands() {
    let (mut session, _) = startup_repl("default", None);
    for command in ["status", "doctor", "history", "memory list"] {
        let repl = execute_repl_line(&mut session, command)
            .expect("repl run")
            .expect("repl frame")
            .content;

        let argv = std::iter::once("bijux".to_string())
            .chain(command.split_whitespace().map(ToString::to_string))
            .collect::<Vec<_>>();
        let core = run_app(&argv).expect("core run");

        if !core.stdout.is_empty() {
            let repl_json: Value = serde_json::from_str(&repl).expect("repl json");
            let core_json: Value = serde_json::from_str(&core.stdout).expect("core json");
            assert_eq!(repl_json, core_json, "divergence for command: {command}");
        }
    }
}
