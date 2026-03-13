#![forbid(unsafe_code)]
//! Best-effort structured telemetry sink for local diagnostics and observability.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

/// Environment variable that enables telemetry writing when set to a file path.
pub const TELEMETRY_FILE_ENV: &str = "BIJUX_TELEMETRY_FILE";
/// Environment variable that allows raw argv capture in telemetry payloads.
pub const TELEMETRY_INCLUDE_ARGS_ENV: &str = "BIJUX_TELEMETRY_INCLUDE_ARGS";

static TELEMETRY_COUNTER: AtomicU64 = AtomicU64::new(1);
static TELEMETRY_WRITE_WARNING_EMITTED: AtomicBool = AtomicBool::new(false);

fn unix_timestamp_millis() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

fn append_json_line(path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(value).map_err(std::io::Error::other)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
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
        let sink_path = std::env::var_os(TELEMETRY_FILE_ENV).map(PathBuf::from);
        let span = Self {
            runtime: runtime.to_string(),
            invocation_id: next_invocation_id(runtime),
            sink_path,
            started_at_ms: unix_timestamp_millis(),
            event_seq: Arc::new(AtomicU64::new(1)),
        };

        let include_args = std::env::var_os(TELEMETRY_INCLUDE_ARGS_ENV).is_some();
        let argv_payload = if include_args {
            json!({ "argv": argv })
        } else {
            json!({
                "argv_count": argv.len(),
                "command_preview": argv.get(1).cloned().unwrap_or_default(),
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
        let event = json!({
            "schema": "bijux-telemetry-event-v1",
            "runtime": self.runtime,
            "pid": process::id(),
            "invocation_id": self.invocation_id,
            "sequence": seq,
            "stage": stage,
            "timestamp_ms": unix_timestamp_millis(),
            "payload": payload,
        });

        if let Err(error) = append_json_line(path, &event) {
            if !TELEMETRY_WRITE_WARNING_EMITTED.swap(true, Ordering::Relaxed) {
                eprintln!("telemetry write failed for {}: {error}", path.to_string_lossy());
            }
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
        self.record(
            "invocation.finish",
            json!({
                "result": "internal_error",
                "exit_code": exit_code,
                "exit_kind": exit_code_kind(exit_code),
                "duration_ms": unix_timestamp_millis().saturating_sub(self.started_at_ms),
                "error_message": error_message,
            }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{exit_code_kind, TelemetrySpan, TELEMETRY_FILE_ENV, TELEMETRY_INCLUDE_ARGS_ENV};
    use serde_json::Value;
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
    }
}
