//! History command handlers.

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::history::operations::{
    clear_history, list_history, HistoryListOptions, DEFAULT_HISTORY_LIMIT, MAX_HISTORY_LIMIT,
};
use crate::routing::parser::root_command;

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
        [a, b] if a == "history" && b == "clear" => {
            let force = parse_history_clear_force(argv)?;
            Ok(Some(clear_history(&paths.history_file, force)?))
        }
        _ => Ok(None),
    }
}

fn parse_history_list_options(argv: &[String]) -> Result<HistoryListOptions> {
    let matches = root_command()
        .try_get_matches_from(argv)
        .map_err(|error| anyhow!("Invalid argument: {error}"))?;
    let Some(history_matches) = matches.subcommand_matches("history") else {
        return Ok(HistoryListOptions::default());
    };

    let mut options = HistoryListOptions {
        limit: history_matches.get_one::<usize>("limit").copied().unwrap_or(DEFAULT_HISTORY_LIMIT),
        filter_contains: history_matches.get_one::<String>("filter").cloned(),
        sort_by_timestamp: history_matches
            .get_one::<String>("sort")
            .is_some_and(|sort| sort == "timestamp"),
    };

    if options.limit == 0 {
        return Err(anyhow!("Invalid argument: --limit must be greater than zero"));
    }
    if options.limit > MAX_HISTORY_LIMIT {
        return Err(anyhow!("Invalid argument: --limit must not exceed {MAX_HISTORY_LIMIT}"));
    }

    if let Some(filter) = options.filter_contains.take() {
        let normalized = filter.trim();
        if normalized.is_empty() {
            return Err(anyhow!("Invalid argument: --filter must not be empty"));
        }
        options.filter_contains = Some(normalized.to_string());
    }

    Ok(options)
}

fn parse_history_clear_force(argv: &[String]) -> Result<bool> {
    let matches = root_command()
        .try_get_matches_from(argv)
        .map_err(|error| anyhow!("Invalid argument: {error}"))?;
    let force = matches
        .subcommand_matches("history")
        .and_then(|history| history.subcommand_matches("clear"))
        .is_some_and(|clear| clear.get_flag("force"));
    Ok(force)
}
