use std::fs;
use std::path::PathBuf;

use crate::infrastructure::fs_store::atomic_write_text;

use super::execution::execute_repl_line;
use super::types::{ReplError, ReplFrame, ReplSession};

fn parse_history_entries(text: &str) -> Option<Vec<String>> {
    if let Ok(entries) = serde_json::from_str::<Vec<String>>(text) {
        return Some(entries);
    }
    if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(text) {
        let commands: Vec<String> = entries
            .into_iter()
            .filter_map(|entry| {
                entry
                    .as_object()
                    .and_then(|obj| obj.get("command"))
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string)
            })
            .collect();
        if !commands.is_empty() {
            return Some(commands);
        }
    }

    let mut parsed = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Python prompt-toolkit history format stores one command per line.
        // We only accept printable command-like lines to avoid treating corrupt
        // blobs as valid history.
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || " :._/-\"'=".contains(ch))
        {
            return None;
        }
        parsed.push(trimmed.to_string());
    }
    Some(parsed)
}

/// Configure history persistence behavior.
pub fn configure_history(
    session: &mut ReplSession,
    history_file: Option<PathBuf>,
    enabled: bool,
    limit: usize,
) {
    session.history_file = history_file;
    session.history_enabled = enabled;
    session.history_limit = limit.max(1);
}

/// Load history into the current session if enabled.
pub fn load_history(session: &mut ReplSession) -> Result<(), ReplError> {
    if !session.history_enabled {
        return Ok(());
    }
    let Some(path) = &session.history_file else {
        return Ok(());
    };
    if !path.exists() {
        return Ok(());
    }

    let text = fs::read_to_string(path)?;
    let mut entries = match parse_history_entries(&text) {
        Some(value) => value,
        None => {
            session.last_error = Some("history file is malformed; history reset".to_string());
            Vec::new()
        }
    };
    if entries.len() > session.history_limit {
        entries = entries.split_off(entries.len() - session.history_limit);
    }
    session.history = entries;
    Ok(())
}

/// Flush history to persistent storage if enabled.
pub fn flush_history(session: &ReplSession) -> Result<(), ReplError> {
    if !session.history_enabled {
        return Ok(());
    }
    let Some(path) = &session.history_file else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_string_pretty(&session.history)?;
    atomic_write_text(path, &(data + "\n"))
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    Ok(())
}

pub(crate) fn push_history(session: &mut ReplSession, command: &str) {
    if !session.history_enabled || command.is_empty() {
        return;
    }
    session.history.push(command.to_string());
    if session.history.len() > session.history_limit {
        let overflow = session.history.len() - session.history_limit;
        session.history.drain(0..overflow);
    }
}

/// Replay a command from history by index.
pub fn replay_history_command(
    session: &mut ReplSession,
    index: usize,
) -> Result<Option<ReplFrame>, ReplError> {
    let command = session
        .history
        .get(index)
        .cloned()
        .ok_or(ReplError::HistoryIndexOutOfBounds(index))?;
    execute_repl_line(session, &command)
}
