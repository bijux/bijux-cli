//! Process entrypoint orchestration for the `bijux-dev-cli` executable.

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use bijux_cli::api::output::{render_value, EmitterConfig};
use bijux_cli::api::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli::api::telemetry::{
    exit_code_kind as telemetry_exit_code_kind, truncate_chars, TelemetrySpan,
    MAX_COMMAND_FIELD_CHARS, MAX_TEXT_FIELD_CHARS,
};
use bijux_cli::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use serde_json::{json, Value};

use crate::cli::dispatch as dev_dispatch;
use crate::runtime::query_provider::RuntimeQueryContext;

/// In-memory process output and exit result produced by the dev-cli runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn decode_os_argv() -> Result<Vec<String>, OsString> {
    let mut argv = Vec::new();
    for value in env::args_os() {
        match value.into_string() {
            Ok(valid) => argv.push(valid),
            Err(invalid) => return Err(invalid),
        }
    }
    Ok(argv)
}

fn emit_run_result(result: &AppRunResult) {
    if !result.stdout.is_empty() {
        let _ = write!(io::stdout(), "{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        let _ = write!(io::stderr(), "{}", result.stderr);
    }
}

fn synthetic_dev_cli_argv(argv: &[String]) -> Vec<String> {
    let mut synthetic = Vec::with_capacity(argv.len().saturating_add(2).max(4));
    synthetic.push("bijux".to_string());
    synthetic.push("dev".to_string());
    synthetic.push("cli".to_string());
    synthetic.extend(argv.iter().skip(1).cloned());
    synthetic
}

fn is_global_flag_without_value(token: &str) -> bool {
    matches!(token, "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text")
}

fn is_global_flag_with_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn is_global_flag_with_equals(token: &str) -> bool {
    token.starts_with("--format=")
        || token.starts_with("--log-level=")
        || token.starts_with("--color=")
        || token.starts_with("--config-path=")
}

fn synthetic_dev_cli_parse_argv(argv: &[String]) -> Vec<String> {
    let mut globals = Vec::new();
    let mut command_tail = Vec::new();

    let mut idx = 1;
    while idx < argv.len() {
        let token = argv[idx].as_str();
        if token == "--" {
            command_tail.extend(argv.iter().skip(idx).cloned());
            break;
        }
        if is_global_flag_without_value(token) || is_global_flag_with_equals(token) {
            globals.push(argv[idx].clone());
            idx += 1;
            continue;
        }
        if is_global_flag_with_value(token) {
            globals.push(argv[idx].clone());
            if let Some(value) = argv.get(idx + 1) {
                globals.push(value.clone());
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }

        command_tail.push(argv[idx].clone());
        idx += 1;
    }

    let mut synthetic = Vec::with_capacity(1 + globals.len() + 2 + command_tail.len());
    synthetic.push("bijux".to_string());
    synthetic.extend(globals);
    synthetic.push("dev".to_string());
    synthetic.push("cli".to_string());
    synthetic.extend(command_tail);
    synthetic
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: no_color_enabled(env::var_os("NO_COLOR")),
    }
}

fn no_color_enabled(value: Option<OsString>) -> bool {
    value.is_some()
}

fn bounded_command(command: &str) -> (String, bool) {
    truncate_chars(command, MAX_COMMAND_FIELD_CHARS)
}

fn bounded_message(message: &str) -> (String, bool) {
    truncate_chars(message, MAX_TEXT_FIELD_CHARS)
}

fn classify_error_exit_code(message: &str) -> i32 {
    let lower = message.to_ascii_lowercase();
    if lower.contains("missing argument")
        || lower.contains("invalid argument")
        || lower.contains("key cannot be empty")
        || lower.contains("invalid key")
        || lower.contains("unknown config section")
        || lower.contains("config key not found")
        || lower.contains("missing parameter")
        || lower.contains("unsupported format")
        || lower.contains("failed to load config")
        || lower.contains("unknown route:")
        || lower.contains("plugin route execution is not implemented")
        || lower.contains("plugin not found")
        || lower.starts_with("invalid format:")
        || lower.starts_with("invalid color mode:")
        || lower.starts_with("invalid log level:")
    {
        2
    } else if lower.contains("non-ascii") || lower.contains("control characters") {
        3
    } else {
        1
    }
}

fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(error.to_string())
        }
        Err(_) => None,
    }
}

fn root_help_text() -> String {
    let help_argv = vec!["bijux".to_string(), "--help".to_string()];
    try_render_clap_help(&help_argv).unwrap_or_default()
}

fn try_render_clap_result(argv: &[String]) -> Option<AppRunResult> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(AppRunResult { exit_code: 0, stdout: error.to_string(), stderr: String::new() })
        }
        Err(error) => {
            let _ = error;
            let message = root_help_text();
            let stderr = if message.ends_with('\n') { message } else { format!("{message}\n") };
            Some(AppRunResult { exit_code: 2, stdout: String::new(), stderr })
        }
    }
}

fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    if !dev_dispatch::owns_path(normalized_path) {
        return Ok(json!({"status": "error", "message": "unknown route"}));
    }

    let context = RuntimeQueryContext::from_flags(global_flags)?;
    let runtime = context.provider();
    let payload = dev_dispatch::try_handle(normalized_path, argv, &runtime)?;

    Ok(payload.unwrap_or_else(|| json!({"status": "error", "message": "unknown route"})))
}

fn expanded_dev_cli_path(normalized_path: &[String], argv: &[String]) -> Vec<String> {
    let [a, b, c] = normalized_path else {
        return normalized_path.to_vec();
    };
    if a != "dev" || b != "cli" {
        return normalized_path.to_vec();
    }
    if !matches!(
        c.as_str(),
        "maintenance" | "rustdoc" | "release" | "evidence" | "config" | "python" | "repo"
    ) {
        return normalized_path.to_vec();
    }

    let Some(start) = argv
        .windows(2)
        .position(|window| window[0] == "dev" && window[1] == "cli")
        .map(|idx| idx + 2)
    else {
        return normalized_path.to_vec();
    };

    let mut expanded = vec!["dev".to_string(), "cli".to_string()];
    for token in argv.iter().skip(start) {
        if token == "--" || token.starts_with('-') {
            break;
        }
        expanded.push(token.clone());
    }

    if expanded.len() < 3 {
        return normalized_path.to_vec();
    }

    expanded
}

fn maintenance_route_exit_code(normalized_path: &[String], payload: &Value) -> Option<i32> {
    let is_maintenance_runner = matches!(
        normalized_path,
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && (d == "generate" || d == "generate-all")
    ) || matches!(
        normalized_path,
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && (e == "run" || e == "run-all")
    );

    if !is_maintenance_runner {
        return None;
    }

    if payload
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status == "failed" || status == "error")
    {
        let exit_code =
            payload.get("exit_code").and_then(Value::as_i64).filter(|code| *code > 0).unwrap_or(1);
        return Some(exit_code as i32);
    }

    if payload.get("failed").and_then(Value::as_u64).is_some_and(|count| count > 0) {
        return Some(1);
    }

    if payload.get("results").and_then(Value::as_array).is_some_and(|rows| {
        rows.iter().any(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "failed" || status == "error")
        })
    }) {
        return Some(1);
    }

    Some(0)
}

fn payload_route_exit_code(normalized_path: &[String], payload: &Value) -> i32 {
    if let Some(code) = payload.get("code").and_then(Value::as_i64).filter(|code| *code > 0) {
        return code as i32;
    }
    if payload.get("status").and_then(Value::as_str) == Some("error") {
        return 1;
    }
    maintenance_route_exit_code(normalized_path, payload).unwrap_or(0)
}

fn normalize_process_exit_code(code: i32) -> u8 {
    if code <= 0 {
        return u8::from(code != 0);
    }
    if code > i32::from(u8::MAX) {
        return u8::MAX;
    }
    code as u8
}

/// Execute `bijux-dev-cli` and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    let telemetry = TelemetrySpan::start("bijux-dev-cli", argv);
    telemetry.record("dispatch.entry", json!({"argv_count": argv.len()}));
    let result = run_app_inner(argv, &telemetry);
    match &result {
        Ok(value) => telemetry.finish_exit(value.exit_code, value.stdout.len(), value.stderr.len()),
        Err(error) => telemetry.finish_internal_error(&error.to_string(), 1),
    }
    result
}

fn run_app_inner(argv: &[String], telemetry: &TelemetrySpan) -> Result<AppRunResult> {
    let synthetic_argv = synthetic_dev_cli_argv(argv);
    let synthetic_parse_argv = synthetic_dev_cli_parse_argv(argv);

    if argv.len() == 1 {
        telemetry.record("dispatch.help.default", json!({"reason":"no_args"}));
        let mut help_argv = synthetic_argv.clone();
        help_argv.push("--help".to_string());
        if let Some(help) = try_render_clap_help(&help_argv) {
            telemetry.record("dispatch.help.rendered", json!({"topic":"root", "exit_code": 0}));
            return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
        }
    }

    if let Some(result) = try_render_clap_result(&synthetic_argv) {
        telemetry.record(
            "dispatch.clap.short_circuit",
            json!({
                "exit_code": result.exit_code,
                "kind": if result.exit_code == 0 { "help_or_version" } else { "usage_error" },
            }),
        );
        return Ok(result);
    }

    let intent = match parse_intent(&synthetic_parse_argv) {
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
    telemetry.record(
        "dispatch.intent.parsed",
        json!({
            "command_path": intent.command_path.clone(),
            "normalized_path": intent.normalized_path.clone(),
            "quiet": intent.global_flags.quiet,
        }),
    );
    if intent.normalized_path.is_empty() {
        telemetry.record("dispatch.intent.empty", json!({}));
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: root_help_text() });
    }

    let expanded_path = expanded_dev_cli_path(&intent.normalized_path, &synthetic_parse_argv);
    let is_unknown = !dev_dispatch::owns_path(&expanded_path);

    let response = route_response(&expanded_path, &synthetic_parse_argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = classify_error_exit_code(&message);
            let command_joined = expanded_path.join(" ");
            let (command, command_truncated) = bounded_command(&command_joined);
            let (message_bounded, message_truncated) = bounded_message(&message);
            telemetry.record(
                "dispatch.route.error",
                json!({
                    "command": command,
                    "command_truncated": command_truncated,
                    "exit_code": code,
                    "exit_kind": telemetry_exit_code_kind(code),
                    "message": message_bounded,
                    "message_truncated": message_truncated,
                }),
            );

            let rendered_error = match render_value(
                &json!({
                    "status": "error",
                    "code": code,
                    "message": message,
                    "command": expanded_path.join(" "),
                }),
                emitter_config(&intent.global_flags),
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

    let rendered = match render_value(&payload, emitter_config(&intent.global_flags)) {
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
        let command_joined = expanded_path.join(" ");
        let (command, command_truncated) = bounded_command(&command_joined);
        telemetry.record(
            "dispatch.route.unknown",
            json!({
                "command": command,
                "command_truncated": command_truncated,
                "exit_code": 2,
                "status": payload.get("status").and_then(Value::as_str),
            }),
        );
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    let route_exit_code = payload_route_exit_code(&expanded_path, &payload);
    let command_joined = expanded_path.join(" ");
    let (command, command_truncated) = bounded_command(&command_joined);
    telemetry.record(
        "dispatch.route.completed",
        json!({
            "command": command,
            "command_truncated": command_truncated,
            "status": payload.get("status").and_then(Value::as_str),
            "exit_code": route_exit_code,
            "exit_kind": telemetry_exit_code_kind(route_exit_code),
        }),
    );

    if intent.global_flags.quiet {
        telemetry.record(
            "dispatch.quiet.suppressed",
            json!({"command": command_joined, "command_truncated": command_joined.chars().count() > MAX_COMMAND_FIELD_CHARS, "exit_code": route_exit_code}),
        );
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult { exit_code: route_exit_code, stdout: content, stderr: String::new() })
}

/// Run `bijux-dev-cli` with current process argv.
#[must_use]
pub fn run_cli_from_env() -> ExitCode {
    let argv = match decode_os_argv() {
        Ok(value) => value,
        Err(_) => {
            let telemetry = TelemetrySpan::start(
                "bijux-dev-cli",
                &["bijux-dev-cli".to_string(), "<invalid-utf8-argv>".to_string()],
            );
            telemetry
                .record("argv.decode.error", json!({"message":"invalid UTF-8 argument in argv"}));
            telemetry.finish_exit(2, 0, "invalid UTF-8 argument in argv\n".len());
            let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
            return ExitCode::from(2);
        }
    };

    let result = match run_app(&argv) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    };

    emit_run_result(&result);
    ExitCode::from(normalize_process_exit_code(result.exit_code))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use bijux_cli::api::telemetry::{TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV};
    use std::ffi::OsString;

    use serde_json::json;

    use super::{
        classify_error_exit_code, expanded_dev_cli_path, no_color_enabled,
        normalize_process_exit_code, payload_route_exit_code, run_app,
        synthetic_dev_cli_parse_argv,
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn parse_argv_lifts_global_flags_before_dev_cli_path() {
        let input = argv(&[
            "bijux-dev-cli",
            "state-audit",
            "--format",
            "text",
            "--config-path",
            "/tmp/config.env",
            "--no-pretty",
        ]);
        let synthetic = synthetic_dev_cli_parse_argv(&input);
        assert_eq!(
            synthetic,
            argv(&[
                "bijux",
                "--format",
                "text",
                "--config-path",
                "/tmp/config.env",
                "--no-pretty",
                "dev",
                "cli",
                "state-audit",
            ])
        );
    }

    #[test]
    fn parse_argv_keeps_command_options_in_tail() {
        let input = argv(&[
            "bijux-dev-cli",
            "maintenance",
            "status",
            "run",
            "--id",
            "STATUS-001",
            "--",
            "--native-flag",
        ]);
        let synthetic = synthetic_dev_cli_parse_argv(&input);
        assert_eq!(
            synthetic,
            argv(&[
                "bijux",
                "dev",
                "cli",
                "maintenance",
                "status",
                "run",
                "--id",
                "STATUS-001",
                "--",
                "--native-flag",
            ])
        );
    }

    #[test]
    fn no_color_is_enabled_by_presence_not_specific_value() {
        assert!(no_color_enabled(Some(OsString::from("1"))));
        assert!(no_color_enabled(Some(OsString::from("0"))));
        assert!(no_color_enabled(Some(OsString::from(""))));
        assert!(!no_color_enabled(None));
    }

    #[test]
    fn expanded_path_lifts_nested_dev_cli_routes_from_argv() {
        let normalized = argv(&["dev", "cli", "maintenance"]);
        let full_argv = argv(&[
            "bijux",
            "--format",
            "json",
            "dev",
            "cli",
            "maintenance",
            "status",
            "run",
            "--id",
            "STATUS-001",
        ]);
        let expanded = expanded_dev_cli_path(&normalized, &full_argv);
        assert_eq!(expanded, argv(&["dev", "cli", "maintenance", "status", "run"]));
    }

    #[test]
    fn expanded_path_keeps_non_nested_routes_stable() {
        let normalized = argv(&["dev", "cli", "state-doctor"]);
        let full_argv = argv(&["bijux", "dev", "cli", "state-doctor", "unexpected"]);
        let expanded = expanded_dev_cli_path(&normalized, &full_argv);
        assert_eq!(expanded, normalized);
    }

    #[test]
    fn payload_error_code_maps_to_non_zero_exit() {
        let code = payload_route_exit_code(
            &argv(&["dev", "cli", "state-doctor"]),
            &json!({"status": "error", "code": 2}),
        );
        assert_eq!(code, 2);
    }

    #[test]
    fn process_exit_code_is_normalized_to_os_range() {
        assert_eq!(normalize_process_exit_code(0), 0);
        assert_eq!(normalize_process_exit_code(2), 2);
        assert_eq!(normalize_process_exit_code(-1), 1);
        assert_eq!(normalize_process_exit_code(300), u8::MAX);
    }

    #[test]
    fn classifier_maps_usage_and_encoding_messages_to_stable_codes() {
        assert_eq!(classify_error_exit_code("Missing argument: --id required"), 2);
        assert_eq!(classify_error_exit_code("invalid format: nope"), 2);
        assert_eq!(
            classify_error_exit_code("Non-ASCII characters are not allowed in keys or values."),
            3
        );
    }

    #[test]
    fn run_app_writes_opt_in_telemetry_events() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let result =
            run_app(&argv(&["bijux-dev-cli", "state-audit", "--format", "json"])).expect("run");
        assert_eq!(result.exit_code, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        assert!(body.lines().any(|line| line.contains("\"stage\":\"invocation.start\"")));
        assert!(body.lines().any(|line| line.contains("\"stage\":\"invocation.finish\"")));
        assert!(body.lines().all(|line| line.contains("\"runtime\":\"bijux-dev-cli\"")));
    }

    #[test]
    fn run_app_unknown_route_emits_unknown_stage_without_completed_stage() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");
        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let result = run_app(&argv(&["bijux-dev-cli", "definitely-not-a-command"])).expect("run");
        assert_eq!(result.exit_code, 2);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry output");
        let rows: Vec<serde_json::Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json")).collect();
        assert!(rows.iter().any(|row| row["stage"] == "dispatch.route.unknown"));
        assert!(
            !rows.iter().any(|row| row["stage"] == "dispatch.route.completed"),
            "unknown routes must not be reported as completed"
        );
    }
}
