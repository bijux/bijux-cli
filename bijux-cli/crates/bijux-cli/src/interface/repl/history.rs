use std::fs;
use std::path::PathBuf;

use crate::infrastructure::fs_store::atomic_write_text;

use super::execution::execute_repl_line;
use super::types::{
    ReplError, ReplFrame, ReplSession, REPL_HISTORY_ENTRY_MAX_CHARS, REPL_HISTORY_FILE_MAX_BYTES,
    REPL_HISTORY_MAX_ENTRIES, REPL_LAST_ERROR_MAX_CHARS,
};

#[derive(Debug, Default)]
struct HistoryParseReport {
    entries: Vec<String>,
    malformed: bool,
    dropped_entries: usize,
    truncated_entries: usize,
}

fn set_history_warning(session: &mut ReplSession, message: &str) {
    let bounded = message
        .chars()
        .filter(|ch| !ch.is_control())
        .take(REPL_LAST_ERROR_MAX_CHARS)
        .collect::<String>();
    session.last_error = Some(bounded);
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
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    if trimmed.is_empty() {
        return HistoryParseReport::default();
    }
    if trimmed.starts_with('[') {
        if let Ok(entries) = serde_json::from_str::<Vec<String>>(trimmed) {
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
        if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
            let mut report = HistoryParseReport::default();
            for entry in entries {
                let normalized = match entry {
                    serde_json::Value::String(value) => sanitize_history_command(&value),
                    serde_json::Value::Object(object) => object
                        .get("command")
                        .and_then(serde_json::Value::as_str)
                        .and_then(sanitize_history_command),
                    _ => None,
                };
                match normalized {
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
        return HistoryParseReport {
            malformed: true,
            dropped_entries: 1,
            ..HistoryParseReport::default()
        };
    }
    if matches!(trimmed.chars().next(), Some('{' | '}' | ']')) {
        return HistoryParseReport {
            malformed: true,
            dropped_entries: 1,
            ..HistoryParseReport::default()
        };
    }

    let mut report = HistoryParseReport::default();
    for line in trimmed.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            let normalized = match value {
                serde_json::Value::String(value) => sanitize_history_command(&value),
                serde_json::Value::Object(object) => object
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .and_then(sanitize_history_command),
                _ => None,
            };
            match normalized {
                Some((sanitized, truncated)) => {
                    report.entries.push(sanitized);
                    report.truncated_entries += usize::from(truncated);
                }
                None => {
                    report.dropped_entries += 1;
                    report.malformed = true;
                }
            }
            continue;
        }
        if matches!(line.chars().next(), Some('[' | ']' | '{' | '}')) {
            report.dropped_entries += 1;
            report.malformed = true;
            continue;
        }
        match sanitize_history_command(line) {
            Some((sanitized, truncated)) => {
                report.entries.push(sanitized);
                report.truncated_entries += usize::from(truncated);
            }
            None => {
                report.dropped_entries += 1;
                report.malformed = true;
            }
        }
    }
    if report.entries.is_empty() && !trimmed.is_empty() {
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
    session.history_limit = limit.clamp(1, REPL_HISTORY_MAX_ENTRIES);
}

/// Load history into the current session if enabled.
pub fn load_history(session: &mut ReplSession) -> Result<(), ReplError> {
    if !session.history_enabled {
        return Ok(());
    }
    let Some(path) = &session.history_file else {
        return Ok(());
    };
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    if metadata.file_type().is_symlink() && fs::metadata(path).is_err() {
        session.history.clear();
        set_history_warning(session, "history path is a broken symlink; history reset");
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        session.history.clear();
        set_history_warning(session, "history path is not a regular file; history reset");
        return Ok(());
    }
    if metadata.len() > REPL_HISTORY_FILE_MAX_BYTES {
        session.history.clear();
        set_history_warning(
            session,
            &format!("history file exceeds {} bytes and was ignored", REPL_HISTORY_FILE_MAX_BYTES),
        );
        return Ok(());
    }

    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            session.history.clear();
            set_history_warning(session, &format!("history file is unreadable: {err}"));
            return Ok(());
        }
    };
    if bytes.len() as u64 > REPL_HISTORY_FILE_MAX_BYTES {
        session.history.clear();
        set_history_warning(
            session,
            &format!("history file exceeds {} bytes and was ignored", REPL_HISTORY_FILE_MAX_BYTES),
        );
        return Ok(());
    }
    let text = match String::from_utf8(bytes) {
        Ok(value) => value,
        Err(_) => {
            session.history.clear();
            set_history_warning(session, "history file is not valid UTF-8; history reset");
            return Ok(());
        }
    };
    let report = parse_history_entries(&text);
    let mut entries = report.entries;
    if entries.len() > session.history_limit {
        entries = entries.split_off(entries.len() - session.history_limit);
    }
    session.history = entries;
    if report.malformed && session.history.is_empty() {
        set_history_warning(session, "history file is malformed; history reset");
    } else if report.dropped_entries > 0 || report.truncated_entries > 0 {
        set_history_warning(
            session,
            &format!(
                "history normalized: dropped={}, truncated={}",
                report.dropped_entries, report.truncated_entries
            ),
        );
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

    let data = {
        let full = serde_json::to_string_pretty(&persisted)?;
        if (full.len() as u64 + 1) <= REPL_HISTORY_FILE_MAX_BYTES {
            full
        } else {
            let mut low = 0usize;
            let mut high = persisted.len();
            while low < high {
                let mid = low + (high - low) / 2;
                let candidate = serde_json::to_string_pretty(&persisted[mid..])?;
                if (candidate.len() as u64 + 1) <= REPL_HISTORY_FILE_MAX_BYTES {
                    high = mid;
                } else {
                    low = mid + 1;
                }
            }
            serde_json::to_string_pretty(&persisted[low..])?
        }
    };
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
        set_history_warning(
            session,
            &format!(
                "history command exceeded {} characters and was truncated",
                REPL_HISTORY_ENTRY_MAX_CHARS
            ),
        );
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
        REPL_HISTORY_ENTRY_MAX_CHARS, REPL_HISTORY_FILE_MAX_BYTES, REPL_HISTORY_MAX_ENTRIES,
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
    fn parse_history_supports_mixed_json_string_and_object_entries() {
        let payload = serde_json::json!([
            "status",
            {"command": "doctor"},
            {"command": "bad\u{0002}"},
            42
        ])
        .to_string();
        let report = parse_history_entries(&payload);
        assert_eq!(report.entries, vec!["status".to_string(), "doctor".to_string()]);
        assert!(report.malformed);
        assert_eq!(report.dropped_entries, 2);
    }

    #[test]
    fn parse_history_fail_closes_on_malformed_structured_payloads() {
        let report = parse_history_entries("{\"command\":\"status\"");
        assert!(report.entries.is_empty());
        assert!(report.malformed);
        assert!(report.dropped_entries > 0);
    }

    #[test]
    fn parse_history_drops_json_shaped_noise_in_line_layout() {
        let report = parse_history_entries("status\n{oops:true}\nplugins list\n");
        assert_eq!(report.entries, vec!["status".to_string(), "plugins list".to_string()]);
        assert!(report.malformed);
        assert_eq!(report.dropped_entries, 1);
    }

    #[test]
    fn parse_history_accepts_utf8_bom_prefixed_json_arrays() {
        let report = parse_history_entries("\u{feff}[\"status\",\"doctor\"]");
        assert_eq!(report.entries, vec!["status".to_string(), "doctor".to_string()]);
        assert!(!report.malformed);
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
    fn load_history_normalizes_invalid_utf8_files() {
        let path = temp_history_file("invalid-utf8");
        std::fs::write(&path, [0xff, 0xfe, 0xfd]).expect("history write should succeed");

        let (mut session, _) = startup_repl("", None);
        configure_history(&mut session, Some(path.clone()), true, 50);
        load_history(&mut session).expect("invalid utf8 should be normalized");

        assert!(session.history.is_empty());
        assert!(session.last_error.as_deref().unwrap_or_default().contains("not valid UTF-8"));

        let _ = std::fs::remove_file(path);
    }

    #[cfg(unix)]
    #[test]
    fn load_history_resets_broken_symlink_paths() {
        use std::os::unix::fs as unix_fs;

        let target = temp_history_file("broken-symlink-target");
        let path = temp_history_file("broken-symlink-link");
        let _ = std::fs::remove_file(&target);
        let _ = std::fs::remove_file(&path);
        unix_fs::symlink(&target, &path).expect("symlink should be created");

        let (mut session, _) = startup_repl("", None);
        configure_history(&mut session, Some(path.clone()), true, 50);
        load_history(&mut session).expect("broken symlink should be normalized");

        assert!(session.history.is_empty());
        assert!(session.last_error.as_deref().unwrap_or_default().contains("broken symlink"));

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

    #[test]
    fn configure_history_clamps_limit_to_max_bound() {
        let (mut session, _) = startup_repl("", None);
        configure_history(
            &mut session,
            Some(temp_history_file("clamp")),
            true,
            REPL_HISTORY_MAX_ENTRIES + 10_000,
        );
        assert_eq!(session.history_limit, REPL_HISTORY_MAX_ENTRIES);
    }

    #[test]
    fn flush_history_never_exceeds_history_file_size_budget() {
        let path = temp_history_file("flush-size-budget");
        let (mut session, _) = startup_repl("", None);
        configure_history(&mut session, Some(path.clone()), true, REPL_HISTORY_MAX_ENTRIES);
        session.history =
            (0..20_000).map(|idx| format!("status {idx} {}", "x".repeat(512))).collect();

        super::flush_history(&session).expect("flush should succeed");
        let metadata = std::fs::metadata(&path).expect("flushed file should exist");
        assert!(metadata.len() <= REPL_HISTORY_FILE_MAX_BYTES);

        let _ = std::fs::remove_file(path);
    }
}
