//! History command handlers.

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::history::operations::{clear_history, list_history, HistoryListOptions};
use crate::shared::argv::command_option_value;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "history" => {
            let list_options = parse_history_list_options(argv)?;
            Ok(Some(list_history(&paths.history_file, &list_options)?))
        }
        [a, b] if a == "history" && b == "clear" => Ok(Some(clear_history(&paths.history_file)?)),
        _ => Ok(None),
    }
}

fn parse_history_list_options(argv: &[String]) -> Result<HistoryListOptions> {
    let mut options = HistoryListOptions::default();

    if let Some(raw) = command_option_value(argv, &["history"], "--limit")
        .or_else(|| command_option_value(argv, &["history"], "-l"))
    {
        options.limit = raw
            .parse::<usize>()
            .map_err(|_| anyhow!("Invalid argument: --limit must be a non-negative integer"))?;
    }
    if let Some(raw) = command_option_value(argv, &["history"], "--filter")
        .or_else(|| command_option_value(argv, &["history"], "-F"))
    {
        options.filter_contains = Some(raw);
    }
    if let Some(sort) = command_option_value(argv, &["history"], "--sort") {
        if sort != "timestamp" {
            return Err(anyhow!(
                "Invalid argument: --sort only supports `timestamp`"
            ));
        }
        options.sort_by_timestamp = true;
    }

    Ok(options)
}
