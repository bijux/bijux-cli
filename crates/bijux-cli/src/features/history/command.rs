//! History command handlers.

use anyhow::Result;
use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::infrastructure::state_store::{read_history_entries, write_history_entries};
use crate::shared::argv::command_positionals;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "history" => {
            let positional = command_positionals(argv, &["history"]);
            let mut limit = 20_usize;
            if let Some(idx) = argv.iter().position(|arg| arg == "--limit" || arg == "-l") {
                if let Some(raw) = argv.get(idx + 1) {
                    limit = raw.parse::<usize>().unwrap_or(20);
                }
            }
            if let Some(raw) = positional.first().and_then(|token| token.strip_prefix("--limit=")) {
                limit = raw.parse::<usize>().unwrap_or(20);
            }
            let mut entries = read_history_entries(&paths.history_file, limit)?;
            if let Some(idx) = argv.iter().position(|arg| arg == "--filter" || arg == "-F") {
                if let Some(needle) = argv.get(idx + 1) {
                    entries.retain(|entry| {
                        entry
                            .get("command")
                            .and_then(Value::as_str)
                            .map(|command| command.contains(needle))
                            .unwrap_or(false)
                    });
                }
            }
            if argv.iter().any(|arg| arg == "--sort")
                && argv
                    .iter()
                    .position(|arg| arg == "--sort")
                    .and_then(|idx| argv.get(idx + 1))
                    .map(|value| value == "timestamp")
                    .unwrap_or(false)
            {
                entries.sort_by(|a, b| {
                    let left = a.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
                    let right = b.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
                    left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            Ok(Some(json!({"entries": entries})))
        }
        [a, b] if a == "history" && b == "clear" => {
            let removed = read_history_entries(&paths.history_file, usize::MAX)
                .map(|entries| entries.len())
                .unwrap_or(0);
            write_history_entries(&paths.history_file, &[])?;
            Ok(Some(
                json!({"status": "cleared", "removed_entries": removed, "file": paths.history_file}),
            ))
        }
        _ => Ok(None),
    }
}
