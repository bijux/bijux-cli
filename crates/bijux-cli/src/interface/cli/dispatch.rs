//! Top-level application entrypoint and route execution.

mod delegation;
mod help;
mod policy;
mod route_exec;
mod suggest;

use anyhow::Result;
use serde_json::json;

use crate::contracts::known_bijux_tools;
use crate::contracts::OutputFormat;
use crate::interface::cli::handlers::install as install_handler;
use crate::interface::cli::help::{render_command_help, render_known_tool_help};
use crate::interface::cli::parser::parse_intent;
use crate::routing::model::{alias_rewrites, built_in_route_paths};
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

fn bounded_command(command: &str) -> (String, bool) {
    truncate_chars(command, MAX_COMMAND_FIELD_CHARS)
}

fn bounded_message(message: &str) -> (String, bool) {
    truncate_chars(message, MAX_TEXT_FIELD_CHARS)
}

fn bounded_status(status: Option<&str>) -> (Option<String>, bool) {
    match status {
        Some(value) => {
            let (bounded, truncated) = bounded_message(value);
            (Some(bounded), truncated)
        }
        None => (None, false),
    }
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

fn levenshtein_distance(left: &str, right: &str) -> usize {
    if left.is_empty() {
        return right.chars().count();
    }
    if right.is_empty() {
        return left.chars().count();
    }

    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut prev = (0..=right_chars.len()).collect::<Vec<usize>>();
    let mut curr = vec![0usize; right_chars.len() + 1];

    for (i, left_ch) in left_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, right_ch) in right_chars.iter().enumerate() {
            let substitution_cost = usize::from(left_ch != right_ch);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + substitution_cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[right_chars.len()]
}

fn known_help_topics() -> Vec<String> {
    let mut topics = built_in_route_paths().to_vec();
    topics.extend(alias_rewrites().iter().map(|(alias, _)| (*alias).to_string()));
    topics.extend(known_bijux_tools().iter().map(|tool| tool.namespace.to_string()));
    for tool in known_bijux_tools() {
        topics.extend(tool.aliases.iter().map(|alias| (*alias).to_string()));
    }
    topics.push("help".to_string());
    let mut expanded = Vec::new();
    for topic in topics {
        let mut prefix = Vec::new();
        for segment in topic.split_whitespace() {
            prefix.push(segment);
            expanded.push(prefix.join(" "));
        }
    }
    expanded.sort();
    expanded.dedup();
    expanded
}

fn is_known_help_topic(path: &[String]) -> bool {
    if path.is_empty() {
        return true;
    }
    let requested = path.join(" ").to_ascii_lowercase();
    known_help_topics().iter().any(|candidate| candidate.eq_ignore_ascii_case(requested.as_str()))
}

fn suggest_help_topics(requested: &str) -> Vec<String> {
    let requested = requested.trim().to_ascii_lowercase();
    if requested.is_empty() {
        return Vec::new();
    }

    let mut scored = Vec::new();
    for candidate in known_help_topics() {
        let normalized = candidate.to_ascii_lowercase();
        if normalized == requested {
            continue;
        }
        let prefix_match =
            normalized.starts_with(&requested) || requested.starts_with(normalized.as_str());
        let distance = levenshtein_distance(&requested, &normalized);
        let threshold = (requested.chars().count().max(normalized.chars().count()) / 3).max(2);
        if prefix_match || distance <= threshold {
            scored.push((!prefix_match, distance, normalized.len(), candidate));
        }
    }

    scored.sort();
    scored.into_iter().map(|(_, _, _, candidate)| candidate).take(3).collect()
}

fn unknown_help_topic_result(requested: &str, telemetry: &TelemetrySpan) -> AppRunResult {
    let suggestions = suggest_help_topics(requested);
    let (requested_bounded, requested_truncated) = bounded_command(requested);
    telemetry.record(
        "dispatch.help.unknown_topic",
        json!({
            "requested": requested_bounded,
            "requested_truncated": requested_truncated,
            "suggestions_count": suggestions.len(),
            "exit_code": 2,
        }),
    );
    let mut stderr = format!("Unknown help topic: {requested}.\n");
    if !suggestions.is_empty() {
        stderr.push_str("Did you mean:\n");
        for suggestion in suggestions {
            stderr.push_str(&format!("  bijux help {suggestion}\n"));
        }
    }
    stderr.push_str("Run `bijux --help` for available runtime commands.\n");
    AppRunResult { exit_code: 2, stdout: String::new(), stderr }
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
        let help_text = match root_usage_help_text() {
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
        let path = match help::parse_help_command_path(argv) {
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
        if let Some(rendered) = render_known_tool_help(&path_refs) {
            let topic = path.join(" ");
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
        if !is_known_help_topic(&path) {
            return Ok(unknown_help_topic_result(&path_refs.join(" "), telemetry));
        }
        let rendered = match render_command_help(&path_refs) {
            Ok(rendered) => rendered,
            Err(_) => return Ok(unknown_help_topic_result(&path_refs.join(" "), telemetry)),
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
    if has_help_flag && delegation::is_known_bijux_tool_route(&argv[1..]) {
        if let Some(delegated) = delegation::try_delegate_known_bijux_tool(argv) {
            let surface = delegation::delegated_command_surface(argv).unwrap_or_default();
            let (target, target_truncated) = bounded_command(&surface);
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
        let surface = delegation::delegated_command_surface(argv).unwrap_or_default();
        let (target, target_truncated) = bounded_command(&surface);
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
    let emitter_config = policy::emitter_config(&intent.global_flags);
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

    if let Some(result) =
        install_handler::try_run(&intent.normalized_path, argv, &intent.global_flags)?
    {
        let command_joined = intent.normalized_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        telemetry.record(
            "dispatch.route.completed",
            json!({
                "command": command,
                "command_truncated": command_truncated,
                "status": if result.exit_code == 0 { Some("ok") } else { Some("error") },
                "status_truncated": false,
                "exit_code": result.exit_code,
                "exit_kind": crate::shared::telemetry::exit_code_kind(result.exit_code),
            }),
        );
        return Ok(result);
    }

    let response = route_exec::route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(route_exec::RouteResponse::Payload(value)) => value,
        Ok(route_exec::RouteResponse::Process(result)) => {
            let command_joined = intent.normalized_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            telemetry.record(
                "dispatch.route.completed",
                json!({
                    "command": command,
                    "command_truncated": command_truncated,
                    "status": if result.exit_code == 0 { Some("ok") } else { Some("error") },
                    "status_truncated": false,
                    "exit_code": result.exit_code,
                    "exit_kind": crate::shared::telemetry::exit_code_kind(result.exit_code),
                }),
            );
            return Ok(result);
        }
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
            let mut suggestion_emitted = false;
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
                    let (nearest_command_bounded, nearest_command_truncated) =
                        bounded_command(&nearest_command);
                    let (next_command_bounded, next_command_truncated) =
                        bounded_command(&next_command);
                    let (next_help_bounded, next_help_truncated) = bounded_command(&next_help);
                    error_payload["nearest_command"] = json!(nearest_command);
                    error_payload["next_command"] = json!(next_command.clone());
                    error_payload["next_help"] = json!(next_help.clone());
                    error_payload["hint"] =
                        json!(format!("Try `{}` or `{}`.", next_command, next_help));
                    suggestion_emitted = true;
                    telemetry.record(
                        "dispatch.route.suggested",
                        json!({
                            "command": command,
                            "command_truncated": command_truncated,
                            "nearest_command": nearest_command_bounded,
                            "nearest_command_truncated": nearest_command_truncated,
                            "next_command": next_command_bounded,
                            "next_command_truncated": next_command_truncated,
                            "next_help": next_help_bounded,
                            "next_help_truncated": next_help_truncated,
                            "source": "error_path",
                        }),
                    );
                }
            }
            if message.starts_with("unknown route: ") {
                telemetry.record(
                    "dispatch.route.unknown",
                    json!({
                        "command": command.clone(),
                        "command_truncated": command_truncated,
                        "exit_code": code,
                        "exit_kind": crate::shared::telemetry::exit_code_kind(code),
                        "source": "error_path",
                        "suggestion_emitted": suggestion_emitted,
                    }),
                );
            }
            let rendered_error = match render_value(&error_payload, emitter_config) {
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

    let rendered = if matches!(intent.normalized_path.as_slice(), [a, b] if a == "cli" && b == "version")
        && emitter_config.format == OutputFormat::Text
    {
        crate::api::version::runtime_version_line()
    } else if matches!(intent.normalized_path.as_slice(), [a, b] if a == "cli" && b == "completion")
        && emitter_config.format == OutputFormat::Text
    {
        payload
            .get("script")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    } else {
        match render_value(&payload, emitter_config) {
            Ok(value) => value,
            Err(error) => {
                let (message, message_truncated) = bounded_message(&error.to_string());
                telemetry.record(
                    "dispatch.render.error",
                    json!({"stream":"stdout","message": message, "message_truncated": message_truncated}),
                );
                return Err(error.into());
            }
        }
    };
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    let route_exit_code = 0;
    let command_joined = intent.normalized_path.join(" ");
    let (command, command_truncated) = bounded_command(&command_joined);
    let (status, status_truncated) =
        bounded_status(payload.get("status").and_then(serde_json::Value::as_str));
    telemetry.record(
        "dispatch.route.completed",
        json!({
            "command": command.clone(),
            "command_truncated": command_truncated,
            "status": status,
            "status_truncated": status_truncated,
            "exit_code": route_exit_code,
            "exit_kind": crate::shared::telemetry::exit_code_kind(route_exit_code),
        }),
    );

    if intent.global_flags.quiet {
        telemetry.record(
            "dispatch.quiet.suppressed",
            json!({
                "command": command,
                "command_truncated": command_truncated,
                "exit_code": route_exit_code,
                "suppressed_stdout_bytes": content.len(),
                "suppressed_stderr_bytes": 0,
            }),
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
    use serde_json::Value;

    use super::{run_app, MAX_PATH_SEGMENT_CHARS};
    use crate::shared::telemetry::{install_test_telemetry_config, TEST_ENV_LOCK};

    #[test]
    fn run_app_writes_opt_in_telemetry_events() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        let _telemetry = install_test_telemetry_config(Some(sink.clone()), false);

        let result = run_app(&["bijux".to_string(), "status".to_string()]).expect("run");
        assert_eq!(result.exit_code, 0);

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
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        let _telemetry = install_test_telemetry_config(Some(sink.clone()), false);

        let result =
            run_app(&["bijux".to_string(), "definitely-not-a-command".to_string()]).expect("run");
        assert_eq!(result.exit_code, 2);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        let unknown = rows
            .iter()
            .find(|row| row["stage"] == "dispatch.route.unknown")
            .expect("unknown route event");
        assert_eq!(unknown["payload"]["exit_kind"], "usage");
        assert_eq!(unknown["payload"]["suggestion_emitted"], true);
        assert_eq!(unknown["payload"]["source"], "error_path");
        assert!(rows.iter().any(|row| row["stage"] == "dispatch.route.suggested"));
        assert!(
            !rows.iter().any(|row| row["stage"] == "dispatch.route.completed"),
            "unknown routes must not be reported as completed"
        );
    }

    #[test]
    fn run_app_quiet_mode_records_suppressed_byte_metrics() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        let _telemetry = install_test_telemetry_config(Some(sink.clone()), false);

        let result = run_app(&["bijux".to_string(), "status".to_string(), "--quiet".to_string()])
            .expect("run");
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.is_empty());
        assert!(result.stderr.is_empty());

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        let suppressed = rows
            .iter()
            .find(|row| row["stage"] == "dispatch.quiet.suppressed")
            .expect("quiet suppressed event");
        assert!(suppressed["payload"]["suppressed_stdout_bytes"].as_u64().unwrap_or_default() > 0);
        assert_eq!(suppressed["payload"]["suppressed_stderr_bytes"], 0);
    }

    #[test]
    fn run_app_bounds_intent_path_segments_in_telemetry() {
        let _guard = TEST_ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        let _telemetry = install_test_telemetry_config(Some(sink.clone()), false);

        let oversized = "x".repeat(MAX_PATH_SEGMENT_CHARS + 48);
        let result = run_app(&["bijux".to_string(), oversized.clone()]).expect("run");
        assert_eq!(result.exit_code, 2);

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

    #[test]
    fn help_unknown_topic_emits_suggestions_for_near_matches() {
        let result =
            run_app(&["bijux".to_string(), "help".to_string(), "sttaus".to_string()]).expect("run");
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("Unknown help topic: sttaus."));
        assert!(result.stderr.contains("Did you mean:"));
        assert!(result.stderr.contains("bijux help status"));
    }

    #[test]
    fn levenshtein_distance_is_deterministic_for_help_suggestions() {
        assert_eq!(super::levenshtein_distance("status", "status"), 0);
        assert_eq!(super::levenshtein_distance("status", "sttaus"), 2);
    }

    #[test]
    fn help_unknown_topic_suggests_root_alias_commands() {
        let result = run_app(&["bijux".to_string(), "help".to_string(), "versoin".to_string()])
            .expect("run");
        assert_eq!(result.exit_code, 2);
        assert!(result.stderr.contains("bijux help version"));
    }

    #[test]
    fn default_help_matches_help_flag_surface() {
        let no_args = run_app(&["bijux".to_string()]).expect("run without args");
        let explicit = run_app(&["bijux".to_string(), "--help".to_string()]).expect("run --help");
        assert_eq!(no_args.exit_code, 0);
        assert_eq!(explicit.exit_code, 0);
        assert_eq!(no_args.stdout, explicit.stdout);
        assert_eq!(no_args.stderr, explicit.stderr);
    }

    #[test]
    fn explain_reports_built_in_route_metadata() {
        let result = run_app(&[
            "bijux".to_string(),
            "explain".to_string(),
            "status".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["route"]["target_class"], "built_in");
        assert_eq!(payload["route"]["owner"], "bijux-cli");
        assert_eq!(payload["route"]["descriptor_source"], "built-in route registry");
        assert_eq!(payload["envelope"]["success_schema"], "output-envelope-v1");
    }

    #[test]
    fn explain_reports_official_app_route_metadata() {
        let result = run_app(&[
            "bijux".to_string(),
            "explain".to_string(),
            "dag".to_string(),
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["route"]["target_class"], "official_app");
        assert_eq!(payload["route"]["owner"], "dag");
        assert_eq!(
            payload["route"]["descriptor_source"],
            "official_product_namespace_registry.json"
        );
        assert_eq!(payload["route"]["side_effect_class"], "external-exec");
    }

    #[test]
    fn explain_reports_unknown_route_metadata() {
        let result = run_app(&[
            "bijux".to_string(),
            "explain".to_string(),
            "definitely-not-a-command".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["status"], "error");
        assert_eq!(payload["route"]["target_class"], "unknown");
    }

    #[test]
    fn cli_script_contract_is_machine_readable() {
        let result = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "script-contract".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["schema_version"], "bijux-cli-script-contract-v1");
        assert_eq!(payload["safe_output_modes"][0], "json");
        assert_eq!(payload["unsafe_for_parsing"][0], "text");
    }

    #[test]
    fn cli_routes_exports_inventory_bundle() {
        let result = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "routes".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["schema_version"], "bijux-cli-route-inventory-v1");
        assert!(payload["inventory"]["builtins"].is_array());
        assert!(payload["inventory"]["aliases"].is_array());
        assert!(payload["inventory"]["namespaces"].is_array());
    }

    #[test]
    fn cli_shims_reports_lifecycle_policy() {
        let result = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "shims".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ])
        .expect("run");
        assert_eq!(result.exit_code, 0);
        let payload: Value = serde_json::from_str(result.stdout.trim()).expect("json");
        assert_eq!(payload["lifecycle_policy"]["shim_support"], "deprecated");
        assert_eq!(payload["lifecycle_policy"]["shadowing_policy"], "refused");
        assert!(payload["legacy_app_shims"].is_array());
    }
}
