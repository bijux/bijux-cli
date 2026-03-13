//! Top-level application entrypoint and route execution.

mod delegation;
mod help;
mod policy;
mod route_exec;
mod suggest;

use anyhow::Result;
use serde_json::json;

use crate::contracts::known_bijux_tool;
use crate::interface::cli::help::render_command_help;
use crate::interface::cli::parser::parse_intent;
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::shared::output::render_value;
use crate::shared::telemetry::{
    truncate_chars, TelemetrySpan, MAX_COMMAND_FIELD_CHARS, MAX_TEXT_FIELD_CHARS,
};

const MAX_PATH_FIELD_SEGMENTS: usize = 32;
const MAX_PATH_SEGMENT_CHARS: usize = 128;

/// In-memory process output and exit result produced by the core app runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn root_usage_help_text() -> Result<String> {
    let help_argv = vec!["bijux".to_string(), "--help".to_string()];
    if let Some(help) = help::try_render_clap_help(&help_argv) {
        return Ok(help);
    }

    Ok(format!("{}\n", render_command_help(&[])?.trim_end()))
}

fn is_known_help_global_flag(token: &str) -> bool {
    matches!(
        token,
        "--help" | "-h" | "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text"
    )
}

fn help_global_flag_takes_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn is_known_help_global_flag_with_equals(token: &str) -> bool {
    token.starts_with("--format=")
        || token.starts_with("--log-level=")
        || token.starts_with("--color=")
        || token.starts_with("--config-path=")
}

fn parse_help_command_path(argv: &[String]) -> std::result::Result<Vec<String>, String> {
    let mut path = Vec::new();
    let mut consume_next = false;

    for token in argv.iter().skip(2) {
        if consume_next {
            consume_next = false;
            continue;
        }

        if token == "--" {
            continue;
        }
        if is_known_help_global_flag(token) || is_known_help_global_flag_with_equals(token) {
            continue;
        }
        if help_global_flag_takes_value(token) {
            consume_next = true;
            continue;
        }
        if token.starts_with('-') {
            return Err(format!("Unknown help flag: {token}"));
        }

        path.push(token.clone());
    }

    Ok(path)
}

fn bounded_command(command: &str) -> (String, bool) {
    truncate_chars(command, MAX_COMMAND_FIELD_CHARS)
}

fn bounded_message(message: &str) -> (String, bool) {
    truncate_chars(message, MAX_TEXT_FIELD_CHARS)
}

fn bounded_segments(path: &[String]) -> (Vec<String>, usize, usize) {
    let mut bounded = Vec::with_capacity(path.len().min(MAX_PATH_FIELD_SEGMENTS));
    let mut truncated_segment_count = 0usize;

    for segment in path.iter().take(MAX_PATH_FIELD_SEGMENTS) {
        let (value, truncated) = truncate_chars(segment, MAX_PATH_SEGMENT_CHARS);
        bounded.push(value);
        if truncated {
            truncated_segment_count += 1;
        }
    }

    let clipped_segment_count = path.len().saturating_sub(MAX_PATH_FIELD_SEGMENTS);
    (bounded, truncated_segment_count, clipped_segment_count)
}

/// Execute the CLI for provided argv and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    let telemetry = TelemetrySpan::start("bijux-cli", argv);
    telemetry.record("dispatch.entry", json!({"argv_count": argv.len()}));
    let result = run_app_inner(argv, &telemetry);
    match &result {
        Ok(value) => telemetry.finish_exit(value.exit_code, value.stdout.len(), value.stderr.len()),
        Err(error) => telemetry.finish_internal_error(&error.to_string(), 1),
    }
    result
}

fn run_app_inner(argv: &[String], telemetry: &TelemetrySpan) -> Result<AppRunResult> {
    if argv.len() == 1 {
        telemetry.record("dispatch.help.default", json!({"reason":"no_args"}));
        let help_text = match render_command_help(&[]) {
            Ok(help) => help,
            Err(error) => {
                let (message, message_truncated) = bounded_message(&error.to_string());
                telemetry.record(
                    "dispatch.help.render.error",
                    json!({"message": message, "message_truncated": message_truncated}),
                );
                return Err(error);
            }
        };
        telemetry.record("dispatch.help.rendered", json!({"topic":"root", "exit_code": 0}));
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", help_text.trim_end()),
            stderr: String::new(),
        });
    }

    if argv.len() == 2 && matches!(argv[1].as_str(), "--version" | "-V") {
        let (flag, flag_truncated) = bounded_command(&argv[1]);
        telemetry.record(
            "dispatch.version.alias",
            json!({"flag": flag, "flag_truncated": flag_truncated}),
        );
        let normalized = vec![argv[0].clone(), "version".to_string()];
        return run_app_inner(&normalized, telemetry);
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path = match parse_help_command_path(argv) {
            Ok(path) => path,
            Err(message) => {
                let (bounded, message_truncated) = bounded_message(&message);
                telemetry.record(
                    "dispatch.help.error",
                    json!({"message": bounded, "message_truncated": message_truncated, "exit_code": 2}),
                );
                let mut stderr = message;
                stderr.push('\n');
                stderr.push_str("Run `bijux --help` for available runtime commands.\n");
                return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr });
            }
        };
        let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
        if let Some(first) = path.first().map(String::as_str) {
            if first == "dev" || known_bijux_tool(first).is_some() {
                let mut delegated_argv = vec!["bijux".to_string()];
                delegated_argv.extend(path.iter().cloned());
                delegated_argv.push("--help".to_string());
                if let Some(delegated) = delegation::try_delegate_known_bijux_tool(&delegated_argv)
                {
                    let (target, target_truncated) = bounded_command(first);
                    telemetry.record(
                        "dispatch.delegated.help",
                        json!({"target": target, "target_truncated": target_truncated, "exit_code": delegated.exit_code}),
                    );
                    return Ok(delegated);
                }
            }
        }
        let rendered = match render_command_help(&path_refs) {
            Ok(rendered) => rendered,
            Err(_) => {
                let requested = path_refs.join(" ");
                let (requested_bounded, requested_truncated) = bounded_command(&requested);
                telemetry.record(
                    "dispatch.help.unknown_topic",
                    json!({
                        "requested": requested_bounded,
                        "requested_truncated": requested_truncated,
                        "exit_code": 2,
                    }),
                );
                return Ok(AppRunResult {
                    exit_code: 2,
                    stdout: String::new(),
                    stderr:
                        "Unknown help topic. Run `bijux --help` for available runtime commands.\n"
                            .to_string(),
                });
            }
        };
        let topic = if path.is_empty() { "root".to_string() } else { path.join(" ") };
        let (topic_bounded, topic_truncated) = bounded_command(&topic);
        telemetry.record(
            "dispatch.help.rendered",
            json!({
                "topic": topic_bounded,
                "topic_truncated": topic_truncated,
                "exit_code": 0,
            }),
        );
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", rendered.trim_end()),
            stderr: String::new(),
        });
    }

    let has_help_flag = argv.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h"));
    if has_help_flag
        && argv.get(1).is_some_and(|first| first == "dev" || known_bijux_tool(first).is_some())
    {
        if let Some(delegated) = delegation::try_delegate_known_bijux_tool(argv) {
            let target_arg = argv.get(1).cloned().unwrap_or_default();
            let (target, target_truncated) = bounded_command(&target_arg);
            telemetry.record(
                "dispatch.delegated.help_flag",
                json!({"target": target, "target_truncated": target_truncated, "exit_code": delegated.exit_code}),
            );
            return Ok(delegated);
        }
    }

    if let Some(help) = help::try_render_clap_help(argv) {
        telemetry.record(
            "dispatch.clap.short_circuit",
            json!({"kind":"help_or_version", "exit_code": 0}),
        );
        return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
    }

    if let Some(delegated) = delegation::try_delegate_known_bijux_tool(argv) {
        let target_arg = argv.get(1).cloned().unwrap_or_default();
        let (target, target_truncated) = bounded_command(&target_arg);
        telemetry.record(
            "dispatch.delegated.command",
            json!({"target": target, "target_truncated": target_truncated, "exit_code": delegated.exit_code}),
        );
        return Ok(delegated);
    }

    if let Some(usage_error) = help::try_render_clap_usage_error(argv) {
        telemetry
            .record("dispatch.clap.short_circuit", json!({"kind":"usage_error", "exit_code": 2}));
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: if usage_error.ends_with('\n') {
                usage_error
            } else {
                format!("{usage_error}\n")
            },
        });
    }

    let intent = match parse_intent(argv) {
        Ok(intent) => intent,
        Err(error) => {
            let (message, message_truncated) = bounded_message(&error.to_string());
            telemetry.record(
                "dispatch.intent.error",
                json!({"message": message, "message_truncated": message_truncated, "exit_code": 2}),
            );
            return Ok(AppRunResult {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{error}\n"),
            });
        }
    };
    let (command_path, command_path_truncated_segment_count, command_path_clipped_segment_count) =
        bounded_segments(&intent.command_path);
    let (
        normalized_path,
        normalized_path_truncated_segment_count,
        normalized_path_clipped_segment_count,
    ) = bounded_segments(&intent.normalized_path);
    telemetry.record(
        "dispatch.intent.parsed",
        json!({
            "command_path": command_path,
            "command_path_truncated_segment_count": command_path_truncated_segment_count,
            "command_path_clipped_segment_count": command_path_clipped_segment_count,
            "normalized_path": normalized_path,
            "normalized_path_truncated_segment_count": normalized_path_truncated_segment_count,
            "normalized_path_clipped_segment_count": normalized_path_clipped_segment_count,
            "quiet": intent.global_flags.quiet,
        }),
    );
    if intent.normalized_path.is_empty() {
        telemetry.record("dispatch.intent.empty", json!({}));
        let usage = match root_usage_help_text() {
            Ok(value) => value,
            Err(error) => {
                let (message, message_truncated) = bounded_message(&error.to_string());
                telemetry.record(
                    "dispatch.help.render.error",
                    json!({"message": message, "message_truncated": message_truncated}),
                );
                return Err(error);
            }
        };
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: usage });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let response = route_exec::route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = policy::classify_error_exit_code(&message);
            let command_joined = intent.normalized_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            let (message_bounded, message_truncated) = bounded_message(&message);
            telemetry.record(
                "dispatch.route.error",
                json!({
                    "command": command.clone(),
                    "command_truncated": command_truncated,
                    "exit_code": code,
                    "exit_kind": crate::shared::telemetry::exit_code_kind(code),
                    "message": message_bounded,
                    "message_truncated": message_truncated,
                }),
            );
            if message.starts_with("unknown route: ") {
                telemetry.record(
                    "dispatch.route.unknown",
                    json!({
                        "command": command.clone(),
                        "command_truncated": command_truncated,
                        "exit_code": code,
                        "source": "error_path",
                    }),
                );
            }
            let mut error_payload = json!({
                "status": "error",
                "code": code,
                "message": message,
                "command": intent.normalized_path.join(" "),
            });
            if message.starts_with("unknown route: ") {
                if let Some(correction) =
                    suggest::correction_for_unknown_route(&intent.normalized_path)
                {
                    let nearest_command = correction.nearest_command;
                    let next_command = correction.next_command;
                    let next_help = correction.next_help;
                    error_payload["nearest_command"] = json!(nearest_command);
                    error_payload["next_command"] = json!(next_command.clone());
                    error_payload["next_help"] = json!(next_help.clone());
                    error_payload["hint"] =
                        json!(format!("Try `{}` or `{}`.", next_command, next_help));
                }
            }
            let rendered_error = match render_value(
                &error_payload,
                policy::emitter_config(&intent.global_flags),
            ) {
                Ok(value) => value,
                Err(error) => {
                    let (message, message_truncated) = bounded_message(&error.to_string());
                    telemetry.record(
                            "dispatch.render.error",
                            json!({"stream":"stderr","message": message, "message_truncated": message_truncated}),
                        );
                    return Err(error.into());
                }
            };
            let error_content = if rendered_error.ends_with('\n') {
                rendered_error
            } else {
                format!("{rendered_error}\n")
            };
            return Ok(AppRunResult {
                exit_code: code,
                stdout: String::new(),
                stderr: error_content,
            });
        }
    };

    let rendered = match render_value(&payload, policy::emitter_config(&intent.global_flags)) {
        Ok(value) => value,
        Err(error) => {
            let (message, message_truncated) = bounded_message(&error.to_string());
            telemetry.record(
                "dispatch.render.error",
                json!({"stream":"stdout","message": message, "message_truncated": message_truncated}),
            );
            return Err(error.into());
        }
    };
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        let command_joined = intent.normalized_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        telemetry.record(
            "dispatch.route.unknown",
            json!({
                "command": command,
                "command_truncated": command_truncated,
                "exit_code": 2,
                "status": payload.get("status").and_then(serde_json::Value::as_str),
            }),
        );
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    let route_exit_code = 0;
    let command_joined = intent.normalized_path.join(" ");
    let (command, command_truncated) = bounded_command(&command_joined);
    telemetry.record(
        "dispatch.route.completed",
        json!({
            "command": command.clone(),
            "command_truncated": command_truncated,
            "status": payload.get("status").and_then(serde_json::Value::as_str),
            "exit_code": route_exit_code,
            "exit_kind": crate::shared::telemetry::exit_code_kind(route_exit_code),
        }),
    );

    if intent.global_flags.quiet {
        telemetry.record(
            "dispatch.quiet.suppressed",
            json!({"command": command, "command_truncated": command_truncated, "exit_code": route_exit_code}),
        );
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult { exit_code: route_exit_code, stdout: content, stderr: String::new() })
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use serde_json::Value;

    use super::{run_app, MAX_PATH_SEGMENT_CHARS};
    use crate::api::telemetry::{TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn run_app_writes_opt_in_telemetry_events() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let result = run_app(&["bijux".to_string(), "status".to_string()]).expect("run");
        assert_eq!(result.exit_code, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        assert!(
            rows.iter().any(|row| row["stage"] == "invocation.start"),
            "telemetry should include invocation.start"
        );
        assert!(
            rows.iter().any(|row| row["stage"] == "invocation.finish"),
            "telemetry should include invocation.finish"
        );
        assert!(rows.iter().all(|row| row["runtime"] == "bijux-cli"));
    }

    #[test]
    fn run_app_unknown_route_emits_unknown_stage_without_completed_stage() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let result =
            run_app(&["bijux".to_string(), "definitely-not-a-command".to_string()]).expect("run");
        assert_eq!(result.exit_code, 2);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        assert!(rows.iter().any(|row| row["stage"] == "dispatch.route.unknown"));
        assert!(
            !rows.iter().any(|row| row["stage"] == "dispatch.route.completed"),
            "unknown routes must not be reported as completed"
        );
    }

    #[test]
    fn run_app_bounds_intent_path_segments_in_telemetry() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let oversized = "x".repeat(MAX_PATH_SEGMENT_CHARS + 48);
        let result = run_app(&["bijux".to_string(), oversized.clone()]).expect("run");
        assert_eq!(result.exit_code, 2);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        let parsed = rows
            .iter()
            .find(|row| row["stage"] == "dispatch.intent.parsed")
            .expect("intent parsed event");
        let first = parsed["payload"]["normalized_path"][0].as_str().expect("first segment");
        assert_eq!(first.chars().count(), MAX_PATH_SEGMENT_CHARS);
        assert_eq!(parsed["payload"]["normalized_path_truncated_segment_count"], 1);
    }
}
