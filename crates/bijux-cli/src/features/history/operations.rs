#![forbid(unsafe_code)]
//! History feature operations.

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::infrastructure::state_store::{read_history_entries, write_history_entries};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryListOptions {
    pub(crate) limit: usize,
    pub(crate) filter_contains: Option<String>,
    pub(crate) sort_by_timestamp: bool,
}

impl Default for HistoryListOptions {
    fn default() -> Self {
        Self { limit: 20, filter_contains: None, sort_by_timestamp: false }
    }
}

pub(crate) fn list_history(history_file: &Path, options: &HistoryListOptions) -> Result<Value> {
    let mut entries = read_history_entries(history_file, options.limit)?;

    if let Some(needle) = options.filter_contains.as_deref() {
        entries.retain(|entry| {
            entry
                .get("command")
                .and_then(Value::as_str)
                .map(|command| command.contains(needle))
                .unwrap_or(false)
        });
    }

    if options.sort_by_timestamp {
        entries.sort_by(|a, b| {
            let left = a.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
            let right = b.get("timestamp").and_then(Value::as_f64).unwrap_or(0.0);
            left.partial_cmp(&right).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    Ok(json!({"entries": entries}))
}

pub(crate) fn clear_history(history_file: &Path) -> Result<Value> {
    let removed =
        read_history_entries(history_file, usize::MAX).map(|entries| entries.len()).unwrap_or(0);
    write_history_entries(history_file, &[])?;

    Ok(json!({"status": "cleared", "removed_entries": removed, "file": history_file}))
}
