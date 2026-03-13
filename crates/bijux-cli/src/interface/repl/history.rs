use std::fs;
use std::path::PathBuf;

use crate::infrastructure::fs_store::atomic_write_text;

use super::execution::execute_repl_line;
use super::types::{
    ReplError, ReplFrame, ReplSession, REPL_HISTORY_ENTRY_MAX_CHARS, REPL_HISTORY_FILE_MAX_BYTES,
};

#[derive(Debug, Default)]
struct HistoryParseReport {
    entries: Vec<String>,
    malformed: bool,
    dropped_entries: usize,
    truncated_entries: usize,
}

fn sanitize_history_command(raw: &str) -> Option<(String, bool)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().any(char::is_control) {
        return None;
    }

    let char_count = trimmed.chars().count();
    if char_count <= REPL_HISTORY_ENTRY_MAX_CHARS {
        return Some((trimmed.to_string(), false));
    }

    let truncated = trimmed.chars().take(REPL_HISTORY_ENTRY_MAX_CHARS).collect::<String>();
    Some((truncated, true))
}

fn parse_history_entries(text: &str) -> HistoryParseReport {
    if let Ok(entries) = serde_json::from_str::<Vec<String>>(text) {
        let mut report = HistoryParseReport::default();
        for entry in entries {
            match sanitize_history_command(&entry) {
                Some((sanitized, truncated)) => {
                    report.entries.push(sanitized);
                    report.truncated_entries += usize::from(truncated);
                }
                None => report.dropped_entries += 1,
            }
        }
        if report.dropped_entries > 0 {
            report.malformed = true;
        }
        return report;
    }
    if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(text) {
        let mut report = HistoryParseReport::default();
        for entry in entries {
            match entry
                .as_object()
                .and_then(|obj| obj.get("command"))
                .and_then(serde_json::Value::as_str)
                .and_then(sanitize_history_command)
            {
                Some((sanitized, truncated)) => {
                    report.entries.push(sanitized);
                    report.truncated_entries += usize::from(truncated);
                }
                None => report.dropped_entries += 1,
            }
        }
        if report.dropped_entries > 0 {
            report.malformed = true;
        }
        return report;
    }

    let mut report = HistoryParseReport::default();
    for line in text.lines() {
        match sanitize_history_command(line) {
            Some((sanitized, truncated)) => {
                report.entries.push(sanitized);
                report.truncated_entries += usize::from(truncated);
            }
            None if !line.trim().is_empty() => {
                report.dropped_entries += 1;
                report.malformed = true;
            }
            None => {}
        }
    }
    if report.entries.is_empty() && !text.trim().is_empty() {
        report.malformed = true;
    }
    report
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
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        session.history.clear();
        session.last_error = Some("history path is not a regular file; history reset".to_string());
        return Ok(());
    }
    if metadata.len() > REPL_HISTORY_FILE_MAX_BYTES {
        session.history.clear();
        session.last_error = Some(format!(
            "history file exceeds {} bytes and was ignored",
            REPL_HISTORY_FILE_MAX_BYTES
        ));
        return Ok(());
    }

    let text = fs::read_to_string(path)?;
    let report = parse_history_entries(&text);
    let mut entries = report.entries;
    if entries.len() > session.history_limit {
        entries = entries.split_off(entries.len() - session.history_limit);
    }
    session.history = entries;
    if report.malformed && session.history.is_empty() {
        session.last_error = Some("history file is malformed; history reset".to_string());
    } else if report.dropped_entries > 0 || report.truncated_entries > 0 {
        session.last_error = Some(format!(
            "history normalized: dropped={}, truncated={}",
            report.dropped_entries, report.truncated_entries
        ));
    } else {
        session.last_error = None;
    }
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

    let mut persisted = session
        .history
        .iter()
        .filter_map(|entry| sanitize_history_command(entry).map(|(sanitized, _)| sanitized))
        .collect::<Vec<_>>();
    if persisted.len() > session.history_limit {
        persisted = persisted.split_off(persisted.len() - session.history_limit);
    }

    let data = serde_json::to_string_pretty(&persisted)?;
    atomic_write_text(path, &(data + "\n"))
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    Ok(())
}

pub(crate) fn push_history(session: &mut ReplSession, command: &str) {
    if !session.history_enabled || command.is_empty() {
        return;
    }
    let Some((sanitized, truncated)) = sanitize_history_command(command) else {
        return;
    };
    session.history.push(sanitized);
    if truncated {
        session.last_error = Some(format!(
            "history command exceeded {} characters and was truncated",
            REPL_HISTORY_ENTRY_MAX_CHARS
        ));
    }
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
    let command =
        session.history.get(index).cloned().ok_or(ReplError::HistoryIndexOutOfBounds(index))?;
    execute_repl_line(session, &command)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        configure_history, load_history, parse_history_entries, push_history,
        REPL_HISTORY_ENTRY_MAX_CHARS, REPL_HISTORY_FILE_MAX_BYTES,
    };
    use crate::interface::repl::session::startup_repl;

    fn temp_history_file(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("bijux-repl-history-{name}-{nanos}.txt"))
    }

    #[test]
    fn parse_history_marks_control_char_lines_as_malformed_and_dropped() {
        let report = parse_history_entries("status\nbad\u{0007}\n");
        assert_eq!(report.entries, vec!["status".to_string()]);
        assert!(report.malformed);
        assert_eq!(report.dropped_entries, 1);
    }

    #[test]
    fn parse_history_truncates_oversized_entries() {
        let long_entry = "x".repeat(REPL_HISTORY_ENTRY_MAX_CHARS + 64);
        let payload = serde_json::to_string(&vec![long_entry]).expect("json serialization");
        let report = parse_history_entries(&payload);

        assert_eq!(report.entries.len(), 1);
        assert_eq!(report.entries[0].chars().count(), REPL_HISTORY_ENTRY_MAX_CHARS);
        assert_eq!(report.truncated_entries, 1);
    }

    #[test]
    fn parse_history_marks_json_entries_with_invalid_commands_as_malformed() {
        let payload = serde_json::to_string(&vec!["status".to_string(), "bad\u{0001}".to_string()])
            .expect("json serialization");
        let report = parse_history_entries(&payload);
        assert_eq!(report.entries, vec!["status".to_string()]);
        assert!(report.malformed);
        assert_eq!(report.dropped_entries, 1);
    }

    #[test]
    fn load_history_reports_normalization_diagnostics() {
        let path = temp_history_file("normalize");
        std::fs::write(&path, "status\nbad\u{0007}\n").expect("history write should succeed");

        let (mut session, _) = startup_repl("", None);
        configure_history(&mut session, Some(path.clone()), true, 50);
        load_history(&mut session).expect("history load should succeed");

        assert_eq!(session.history, vec!["status".to_string()]);
        assert!(session.last_error.as_deref().unwrap_or_default().contains("history normalized"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn push_history_truncates_entries_to_bounded_size() {
        let (mut session, _) = startup_repl("", None);
        let long_entry = "x".repeat(REPL_HISTORY_ENTRY_MAX_CHARS + 64);

        push_history(&mut session, &long_entry);

        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].chars().count(), REPL_HISTORY_ENTRY_MAX_CHARS);
        assert!(session
            .last_error
            .as_deref()
            .unwrap_or_default()
            .contains("history command exceeded"));
    }

    #[test]
    fn load_history_ignores_oversized_files() {
        let path = temp_history_file("oversized");
        let oversized = vec![b'x'; (REPL_HISTORY_FILE_MAX_BYTES + 1024) as usize];
        std::fs::write(&path, oversized).expect("history write should succeed");

        let (mut session, _) = startup_repl("", None);
        session.last_error = Some("stale".to_string());
        configure_history(&mut session, Some(path.clone()), true, 50);
        load_history(&mut session).expect("history load should succeed");

        assert!(session.history.is_empty());
        assert!(session.last_error.as_deref().unwrap_or_default().contains("exceeds"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_history_clears_previous_error_after_clean_read() {
        let path = temp_history_file("clean-read");
        std::fs::write(&path, "status\n").expect("history write should succeed");

        let (mut session, _) = startup_repl("", None);
        session.last_error = Some("stale".to_string());
        configure_history(&mut session, Some(path.clone()), true, 50);
        load_history(&mut session).expect("history load should succeed");

        assert_eq!(session.history, vec!["status".to_string()]);
        assert!(session.last_error.is_none());

        let _ = std::fs::remove_file(path);
    }
}
