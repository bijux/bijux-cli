//! Root command handlers.

use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

pub(crate) fn try_handle(normalized_path: &[String], argv: &[String]) -> Option<Value> {
    match normalized_path {
        [a] if a == "status" => Some(json!({"status": "ok", "runtime": "rust-foundation"})),
        [a] if a == "audit" => Some(json!({
            "status": "ok",
            "checks": ["config", "paths", "plugins", "history"],
            "issues": []
        })),
        [a] if a == "docs" => Some(json!({
            "status": "ok",
            "topics": ["commands", "configuration", "plugins", "repl", "diagnostics"],
        })),
        [a] if a == "sleep" => {
            let duration_secs = argv
                .get(2)
                .and_then(|raw| raw.parse::<f64>().ok())
                .map(|v| v.clamp(0.0, 2.0))
                .unwrap_or(0.0);
            if duration_secs > 0.0 {
                thread::sleep(Duration::from_secs_f64(duration_secs));
            }
            Some(json!({"status": "ok", "slept_seconds": duration_secs}))
        }
        _ => None,
    }
}
