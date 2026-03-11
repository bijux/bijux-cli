//! History command handlers.

use anyhow::Result;
use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::history::operations::{clear_history, list_history, HistoryListOptions};
use crate::shared::argv::command_positionals;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "history" => {
            let list_options = parse_history_list_options(argv);
            Ok(Some(list_history(&paths.history_file, &list_options)?))
        }
        [a, b] if a == "history" && b == "clear" => Ok(Some(clear_history(&paths.history_file)?)),
        _ => Ok(None),
    }
}

fn parse_history_list_options(argv: &[String]) -> HistoryListOptions {
    let positional = command_positionals(argv, &["history"]);
    let mut options = HistoryListOptions::default();

    if let Some(idx) = argv.iter().position(|arg| arg == "--limit" || arg == "-l") {
        if let Some(raw) = argv.get(idx + 1) {
            options.limit = raw.parse::<usize>().unwrap_or(options.limit);
        }
    }
    if let Some(raw) = positional
        .first()
        .and_then(|token| token.strip_prefix("--limit="))
    {
        options.limit = raw.parse::<usize>().unwrap_or(options.limit);
    }
    if let Some(idx) = argv.iter().position(|arg| arg == "--filter" || arg == "-F") {
        options.filter_contains = argv.get(idx + 1).cloned();
    }
    if argv.iter().any(|arg| arg == "--sort")
        && argv
            .iter()
            .position(|arg| arg == "--sort")
            .and_then(|idx| argv.get(idx + 1))
            .map(|value| value == "timestamp")
            .unwrap_or(false)
    {
        options.sort_by_timestamp = true;
    }

    options
}
