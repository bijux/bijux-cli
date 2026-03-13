#![forbid(unsafe_code)]
//! Best-effort structured telemetry sink for local diagnostics and observability.

use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

/// Environment variable that enables telemetry writing when set to a file path.
pub const TELEMETRY_FILE_ENV: &str = "BIJUX_TELEMETRY_FILE";
/// Environment variable that allows raw argv capture in telemetry payloads.
pub const TELEMETRY_INCLUDE_ARGS_ENV: &str = "BIJUX_TELEMETRY_INCLUDE_ARGS";

static TELEMETRY_COUNTER: AtomicU64 = AtomicU64::new(1);
static TELEMETRY_CONFIG_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);
static TELEMETRY_CONFIG_WARNING_KEYS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static TELEMETRY_WRITE_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);
static TELEMETRY_WRITE_WARNING_KEYS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
const MAX_COMMAND_PREVIEW_CHARS: usize = 128;
const MAX_ARG_CHARS: usize = 256;
const MAX_CAPTURED_ARGS: usize = 64;
const MAX_STAGE_FIELD_CHARS: usize = 128;
/// Max number of chars retained for telemetry message-like fields.
pub const MAX_TEXT_FIELD_CHARS: usize = 2048;
/// Max number of chars retained for telemetry command-like fields.
pub const MAX_COMMAND_FIELD_CHARS: usize = 512;

fn unix_timestamp_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

fn append_json_line(path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut line = serde_json::to_vec(value).map_err(std::io::Error::other)?;
    line.push(b'\n');
    file.write_all(&line)?;
    Ok(())
}

fn emit_telemetry_config_warning_once(message: &str) {
    let key = message.to_string();
    let cache = TELEMETRY_CONFIG_WARNING_KEYS.get_or_init(|| Mutex::new(HashSet::new()));
    let should_emit = match cache.lock() {
        Ok(mut seen) => seen.insert(key),
        Err(_) => !TELEMETRY_CONFIG_WARNING_EMITTED.swap(true, Ordering::Relaxed),
    };
    if should_emit {
        eprintln!("{message}");
    }
}

/// Truncate a text field to a stable char budget.
#[must_use]
pub fn truncate_chars(input: &str, limit: usize) -> (String, bool) {
    let total = input.chars().count();
    if total <= limit {
        return (input.to_string(), false);
    }
    (input.chars().take(limit).collect(), true)
}

fn emit_telemetry_write_warning_once(path: &Path, error: &std::io::Error) {
    let key = format!(
        "{}|{:?}|{:?}",
        path.to_string_lossy(),
        error.kind(),
        error.raw_os_error()
    );
    let cache = TELEMETRY_WRITE_WARNING_KEYS.get_or_init(|| Mutex::new(HashSet::new()));
    let should_emit = match cache.lock() {
        Ok(mut seen) => seen.insert(key),
        Err(_) => !TELEMETRY_WRITE_WARNING_EMITTED.swap(true, Ordering::Relaxed),
    };
    if should_emit {
        eprintln!("telemetry write failed for {}: {error}", path.to_string_lossy());
    }
}

fn sanitize_argv(argv: &[String]) -> Value {
    let mut args = Vec::new();
    let mut truncated_args = 0usize;
    let mut clipped_by_count = 0usize;

    for value in argv.iter().take(MAX_CAPTURED_ARGS) {
        let (sanitized, truncated) = truncate_chars(value, MAX_ARG_CHARS);
        args.push(sanitized);
        if truncated {
            truncated_args += 1;
        }
    }

    if argv.len() > MAX_CAPTURED_ARGS {
        clipped_by_count = argv.len() - MAX_CAPTURED_ARGS;
    }

    json!({
        "argv": args,
        "argv_total_count": argv.len(),
        "argv_truncated_arg_count": truncated_args,
        "argv_clipped_count": clipped_by_count,
    })
}

fn resolve_sink_path() -> Option<PathBuf> {
    let raw = std::env::var_os(TELEMETRY_FILE_ENV)?;
    let raw_path = PathBuf::from(raw);
    let display = raw_path.to_string_lossy().to_string();

    if display.trim().is_empty() {
        emit_telemetry_config_warning_once(
            "telemetry disabled: BIJUX_TELEMETRY_FILE is empty or whitespace",
        );
        return None;
    }

    let candidate = if raw_path.is_absolute() {
        raw_path
    } else {
        match std::env::current_dir() {
            Ok(cwd) => cwd.join(raw_path),
            Err(error) => {
                emit_telemetry_config_warning_once(&format!(
                    "telemetry disabled: failed to resolve BIJUX_TELEMETRY_FILE against cwd: {error}"
                ));
                return None;
            }
        }
    };

    if candidate.is_dir() {
        emit_telemetry_config_warning_once(&format!(
            "telemetry disabled: BIJUX_TELEMETRY_FILE points to a directory ({})",
            candidate.to_string_lossy()
        ));
        return None;
    }

    Some(candidate)
}

fn next_invocation_id(runtime: &str) -> String {
    let seq = TELEMETRY_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{runtime}-{}-{}-{seq}", process::id(), unix_timestamp_millis())
}

/// Stable exit-kind mapping used in telemetry records.
#[must_use]
pub fn exit_code_kind(code: i32) -> &'static str {
    match code {
        0 => "success",
        2 => "usage",
        3 => "encoding",
        130 => "aborted",
        _ => "error",
    }
}

/// In-memory telemetry span that writes line-delimited JSON events when enabled.
#[derive(Debug, Clone)]
pub struct TelemetrySpan {
    runtime: String,
    invocation_id: String,
    sink_path: Option<PathBuf>,
    started_at_ms: u128,
    event_seq: Arc<AtomicU64>,
}

impl TelemetrySpan {
    /// Start a telemetry span for one CLI invocation.
    #[must_use]
    pub fn start(runtime: &str, argv: &[String]) -> Self {
        let sink_path = resolve_sink_path();
        let span = Self {
            runtime: runtime.to_string(),
            invocation_id: next_invocation_id(runtime),
            sink_path,
            started_at_ms: unix_timestamp_millis(),
            event_seq: Arc::new(AtomicU64::new(1)),
        };

        let include_args = std::env::var_os(TELEMETRY_INCLUDE_ARGS_ENV).is_some();
        let argv_payload = if include_args {
            sanitize_argv(argv)
        } else {
            let (command_preview, truncated) =
                truncate_chars(argv.get(1).map_or("", String::as_str), MAX_COMMAND_PREVIEW_CHARS);
            json!({
                "argv_count": argv.len(),
                "command_preview": command_preview,
                "command_preview_truncated": truncated,
            })
        };
        span.record("invocation.start", argv_payload);
        span
    }

    /// Record an intermediate telemetry event.
    pub fn record(&self, stage: &str, payload: Value) {
        let Some(path) = &self.sink_path else {
            return;
        };

        let seq = self.event_seq.fetch_add(1, Ordering::Relaxed);
        let (stage_name, stage_truncated) = truncate_chars(stage, MAX_STAGE_FIELD_CHARS);
        let event = json!({
            "schema": "bijux-telemetry-event-v1",
            "runtime": self.runtime,
            "pid": process::id(),
            "invocation_id": self.invocation_id,
            "sequence": seq,
            "stage": stage_name,
            "stage_truncated": stage_truncated,
            "timestamp_ms": unix_timestamp_millis(),
            "elapsed_ms": unix_timestamp_millis().saturating_sub(self.started_at_ms),
            "payload": payload,
        });

        if let Err(error) = append_json_line(path, &event) {
            emit_telemetry_write_warning_once(path, &error);
        }
    }

    /// Record invocation completion based on final process exit.
    pub fn finish_exit(&self, exit_code: i32, stdout_bytes: usize, stderr_bytes: usize) {
        self.record(
            "invocation.finish",
            json!({
                "result": if exit_code == 0 { "ok" } else { "nonzero_exit" },
                "exit_code": exit_code,
                "exit_kind": exit_code_kind(exit_code),
                "stdout_bytes": stdout_bytes,
                "stderr_bytes": stderr_bytes,
                "duration_ms": unix_timestamp_millis().saturating_sub(self.started_at_ms),
            }),
        );
    }

    /// Record invocation failure caused by internal runtime errors.
    pub fn finish_internal_error(&self, error_message: &str, exit_code: i32) {
        let (message, message_truncated) = truncate_chars(error_message, MAX_TEXT_FIELD_CHARS);
        self.record(
            "invocation.finish",
            json!({
                "result": "internal_error",
                "exit_code": exit_code,
                "exit_kind": exit_code_kind(exit_code),
                "duration_ms": unix_timestamp_millis().saturating_sub(self.started_at_ms),
                "error_message": message,
                "error_message_truncated": message_truncated,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        exit_code_kind, truncate_chars, TelemetrySpan, MAX_CAPTURED_ARGS, MAX_STAGE_FIELD_CHARS,
        MAX_TEXT_FIELD_CHARS, TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV,
    };
    use serde_json::{json, Value};
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn exit_code_kind_maps_stable_classes() {
        assert_eq!(exit_code_kind(0), "success");
        assert_eq!(exit_code_kind(2), "usage");
        assert_eq!(exit_code_kind(3), "encoding");
        assert_eq!(exit_code_kind(130), "aborted");
        assert_eq!(exit_code_kind(1), "error");
        assert_eq!(exit_code_kind(77), "error");
    }

    #[test]
    fn span_writes_start_and_finish_events_when_enabled() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("telemetry").join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let argv = vec!["bijux".to_string(), "status".to_string()];
        let span = TelemetrySpan::start("bijux-cli", &argv);
        span.record("intent.parsed", serde_json::json!({"normalized_path":"status"}));
        span.finish_exit(0, 11, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json line")).collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["stage"], "invocation.start");
        assert_eq!(rows[1]["stage"], "intent.parsed");
        assert_eq!(rows[2]["stage"], "invocation.finish");
        assert_eq!(rows[2]["payload"]["exit_kind"], "success");
        assert_eq!(rows[2]["payload"]["result"], "ok");
        assert_eq!(rows[0]["runtime"], "bijux-cli");
        assert_eq!(rows[0]["sequence"], 1);
        assert_eq!(rows[1]["sequence"], 2);
        assert_eq!(rows[2]["sequence"], 3);
        assert!(rows.iter().all(|row| row["elapsed_ms"].is_number()));
        assert!(rows.iter().all(|row| row["stage_truncated"].is_boolean()));
    }

    #[test]
    fn span_marks_non_zero_exit_as_nonzero_result() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let argv = vec!["bijux".to_string(), "status".to_string()];
        let span = TelemetrySpan::start("bijux-cli", &argv);
        span.finish_exit(2, 0, 42);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json line")).collect();
        assert_eq!(rows[1]["payload"]["result"], "nonzero_exit");
        assert_eq!(rows[1]["payload"]["exit_kind"], "usage");
    }

    #[test]
    fn span_can_include_raw_argv_when_opted_in() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::set_var(TELEMETRY_INCLUDE_ARGS_ENV, "1");

        let argv = vec!["bijux".to_string(), "config".to_string(), "list".to_string()];
        let span = TelemetrySpan::start("bijux-cli", &argv);
        span.finish_internal_error("boom", 1);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let first: Value =
            serde_json::from_str(body.lines().next().expect("first line")).expect("json line");
        assert_eq!(first["payload"]["argv"][0], "bijux");
        assert_eq!(first["payload"]["argv"][2], "list");
        assert_eq!(first["payload"]["argv_total_count"], 3);
        assert_eq!(first["payload"]["argv_clipped_count"], 0);
    }

    #[test]
    fn span_truncates_captured_argv_when_opted_in() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::set_var(TELEMETRY_INCLUDE_ARGS_ENV, "1");

        let mut argv = vec!["bijux".to_string()];
        argv.extend((0..70).map(|idx| format!("arg-{idx}-{}", "x".repeat(300))));
        let span = TelemetrySpan::start("bijux-cli", &argv);
        span.finish_exit(0, 0, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let first: Value =
            serde_json::from_str(body.lines().next().expect("first line")).expect("json line");
        let args = first["payload"]["argv"].as_array().expect("argv array");
        assert_eq!(args.len(), MAX_CAPTURED_ARGS);
        assert_eq!(first["payload"]["argv_total_count"], 71);
        assert_eq!(first["payload"]["argv_clipped_count"], 7);
        assert!(first["payload"]["argv_truncated_arg_count"].as_u64().unwrap_or_default() > 0);
    }

    #[test]
    fn span_disables_sink_when_path_is_whitespace() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        std::env::set_var(TELEMETRY_FILE_ENV, "   ");
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let span = TelemetrySpan::start("bijux-cli", &["bijux".to_string()]);
        span.finish_exit(0, 0, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);
    }

    #[test]
    fn span_disables_sink_when_path_points_to_directory() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        std::env::set_var(TELEMETRY_FILE_ENV, temp.path());
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let span = TelemetrySpan::start("bijux-cli", &["bijux".to_string()]);
        span.finish_exit(0, 0, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);
    }

    #[test]
    fn span_truncates_internal_error_message() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let message = "e".repeat(MAX_TEXT_FIELD_CHARS + 100);
        let span = TelemetrySpan::start("bijux-cli", &["bijux".to_string()]);
        span.finish_internal_error(&message, 1);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let finish: Value =
            serde_json::from_str(body.lines().nth(1).expect("finish line")).expect("json line");
        let rendered = finish["payload"]["error_message"].as_str().unwrap_or_default();
        assert_eq!(rendered.chars().count(), MAX_TEXT_FIELD_CHARS);
        assert_eq!(finish["payload"]["error_message_truncated"], true);
    }

    #[test]
    fn truncate_chars_reports_when_input_is_trimmed() {
        let input = "abcde";
        let (value, truncated) = truncate_chars(input, 3);
        assert_eq!(value, "abc");
        assert!(truncated);

        let (value, truncated) = truncate_chars(input, 5);
        assert_eq!(value, "abcde");
        assert!(!truncated);
    }

    #[test]
    fn span_truncates_oversized_stage_names() {
        let _guard = ENV_LOCK.lock().expect("env lock");
        let temp = tempfile::tempdir().expect("temp dir");
        let sink = temp.path().join("events.jsonl");

        std::env::set_var(TELEMETRY_FILE_ENV, &sink);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let span = TelemetrySpan::start("bijux-cli", &["bijux".to_string()]);
        span.record(&"s".repeat(MAX_STAGE_FIELD_CHARS + 32), json!({"ok": true}));
        span.finish_exit(0, 0, 0);

        std::env::remove_var(TELEMETRY_FILE_ENV);
        std::env::remove_var(TELEMETRY_INCLUDE_ARGS_ENV);

        let body = std::fs::read_to_string(&sink).expect("telemetry body");
        let rows: Vec<Value> =
            body.lines().map(|line| serde_json::from_str(line).expect("json line")).collect();
        let oversized = rows.iter().find(|row| row["payload"]["ok"] == true).expect("oversized row");
        let stage = oversized["stage"].as_str().expect("stage");
        assert_eq!(stage.chars().count(), MAX_STAGE_FIELD_CHARS);
        assert_eq!(oversized["stage_truncated"], true);
    }
}
