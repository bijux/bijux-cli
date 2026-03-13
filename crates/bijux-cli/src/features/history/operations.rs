#![forbid(unsafe_code)]
//! History feature operations.

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::infrastructure::state_store::{read_history_report, write_history_entries};

pub(crate) const DEFAULT_HISTORY_LIMIT: usize = 20;
pub(crate) const MAX_HISTORY_LIMIT: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryListOptions {
    pub(crate) limit: usize,
    pub(crate) filter_contains: Option<String>,
    pub(crate) sort_by_timestamp: bool,
}

impl Default for HistoryListOptions {
    fn default() -> Self {
        Self { limit: DEFAULT_HISTORY_LIMIT, filter_contains: None, sort_by_timestamp: false }
    }
}

pub(crate) fn list_history(history_file: &Path, options: &HistoryListOptions) -> Result<Value> {
    let limit = options.limit.clamp(1, MAX_HISTORY_LIMIT);
    let report = read_history_report(history_file, usize::MAX)?;
    let total_entries = report.entries.len();
    let mut indexed = report.entries.into_iter().enumerate().collect::<Vec<_>>();

    if let Some(needle) = options.filter_contains.as_deref() {
        indexed.retain(|(_, entry)| {
            entry
                .get("command")
                .and_then(Value::as_str)
                .map(|command| command.contains(needle))
                .unwrap_or(false)
        });
    }

    if options.sort_by_timestamp {
        indexed.sort_by(|(left_idx, left), (right_idx, right)| {
            let left_timestamp =
                left.get("timestamp").and_then(Value::as_f64).unwrap_or(f64::NEG_INFINITY);
            let right_timestamp =
                right.get("timestamp").and_then(Value::as_f64).unwrap_or(f64::NEG_INFINITY);
            left_timestamp.total_cmp(&right_timestamp).then_with(|| left_idx.cmp(right_idx))
        });
    }

    if indexed.len() > limit {
        indexed = indexed.split_off(indexed.len() - limit);
    }
    let entries = indexed.into_iter().map(|(_, entry)| entry).collect::<Vec<_>>();

    Ok(json!({
        "entries": entries,
        "summary": {
            "source_format": report.source_format,
            "file_bytes": report.file_bytes,
            "total_entries": total_entries,
            "returned_entries": entries.len(),
            "dropped_invalid_entries": report.dropped_invalid_entries,
            "limit": limit,
            "filter_applied": options.filter_contains.is_some(),
            "sort_mode": if options.sort_by_timestamp { "timestamp" } else { "preserve" },
        }
    }))
}

pub(crate) fn clear_history(history_file: &Path) -> Result<Value> {
    let report = read_history_report(history_file, usize::MAX)?;
    let removed = report.entries.len();
    write_history_entries(history_file, &[])?;

    Ok(json!({
        "status": "cleared",
        "removed_entries": removed,
        "dropped_invalid_entries": report.dropped_invalid_entries,
        "source_format": report.source_format,
        "file": history_file,
    }))
}
