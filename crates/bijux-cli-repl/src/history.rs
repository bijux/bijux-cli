use std::fs;
use std::path::PathBuf;

use crate::execution::execute_repl_line;
use crate::types::{ReplError, ReplFrame, ReplSession};

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
    let mut entries: Vec<String> = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(_) => {
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
    fs::write(path, format!("{data}\n"))?;
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
