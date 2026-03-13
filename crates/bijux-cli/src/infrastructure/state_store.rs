//! State-file persistence helpers shared by command handlers.

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde_json::{json, Value};

use crate::infrastructure::fs_store::atomic_write_text;

const MAX_HISTORY_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_HISTORY_COMMAND_CHARS: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryReadReport {
    pub(crate) entries: Vec<Value>,
    pub(crate) source_format: &'static str,
    pub(crate) dropped_invalid_entries: usize,
    pub(crate) file_bytes: u64,
}

fn normalize_history_command(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_control) {
        return None;
    }
    Some(trimmed.chars().take(MAX_HISTORY_COMMAND_CHARS).collect())
}

fn history_entry_from_command(command: &str) -> Option<Value> {
    let command = normalize_history_command(command)?;
    json!({
        "command": command,
        "params": [],
        "timestamp": 0.0,
        "success": true,
        "return_code": 0,
        "duration_ms": 0.0,
        "raw": {},
    })
    .into()
}

fn normalize_history_object(mut value: Value) -> Option<Value> {
    let object = value.as_object_mut()?;
    let command = object.get("command")?.as_str()?;
    let normalized = normalize_history_command(command)?;
    object.insert("command".to_string(), Value::String(normalized));
    Some(value)
}

fn push_history_entry(entries: &mut VecDeque<Value>, entry: Value, limit: usize) {
    if limit == 0 {
        return;
    }
    if limit != usize::MAX {
        while entries.len() >= limit {
            entries.pop_front();
        }
    }
    entries.push_back(entry);
}

struct LimitedHistoryArraySeed {
    limit: usize,
}

impl<'de> DeserializeSeed<'de> for LimitedHistoryArraySeed {
    type Value = ParsedHistoryEntries;

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(LimitedHistoryArrayVisitor { limit: self.limit })
    }
}

struct LimitedHistoryArrayVisitor {
    limit: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct ParsedHistoryEntries {
    entries: Vec<Value>,
    dropped_invalid_entries: usize,
}

impl<'de> Visitor<'de> for LimitedHistoryArrayVisitor {
    type Value = ParsedHistoryEntries;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON array of history entries")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entries = VecDeque::<Value>::new();
        let mut dropped_invalid_entries = 0usize;
        while let Some(value) = seq.next_element::<Value>()? {
            if let Some(normalized) = normalize_history_object(value) {
                push_history_entry(&mut entries, normalized, self.limit);
            } else {
                dropped_invalid_entries += 1;
            }
        }
        Ok(ParsedHistoryEntries { entries: entries.into_iter().collect(), dropped_invalid_entries })
    }
}

fn parse_history_array_entries(text: &str, limit: usize) -> Result<ParsedHistoryEntries> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let parsed = LimitedHistoryArraySeed { limit }
        .deserialize(&mut deserializer)
        .map_err(|err| anyhow::anyhow!(err))?;
    deserializer.end().map_err(|err| anyhow::anyhow!(err))?;
    Ok(parsed)
}

fn parse_history_entries(text: &str, limit: usize) -> Result<HistoryReadReport> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(HistoryReadReport {
            entries: Vec::new(),
            source_format: "empty",
            dropped_invalid_entries: 0,
            file_bytes: 0,
        });
    }

    if trimmed.starts_with('[') {
        let parsed = parse_history_array_entries(trimmed, limit).map_err(|error| {
            anyhow::anyhow!("Malformed history state: invalid JSON array payload: {error}")
        })?;
        return Ok(HistoryReadReport {
            entries: parsed.entries,
            source_format: "json-array",
            dropped_invalid_entries: parsed.dropped_invalid_entries,
            file_bytes: 0,
        });
    }

    if trimmed.starts_with('{') {
        if serde_json::from_str::<Value>(trimmed).is_ok() {
            anyhow::bail!("Malformed history state: expected JSON array payload");
        }
        anyhow::bail!("Malformed history state: invalid JSON object payload");
    }

    if matches!(trimmed.chars().next(), Some(']' | '}')) {
        anyhow::bail!("Malformed history state: invalid JSON payload");
    }

    // Compatibility fallback for line-oriented history files with partial corruption.
    let mut out = VecDeque::<Value>::new();
    let mut dropped_invalid_entries = 0usize;
    let mut saw_json_line = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            saw_json_line = true;
            if let Some(normalized) = normalize_history_object(value) {
                push_history_entry(&mut out, normalized, limit);
            } else {
                dropped_invalid_entries += 1;
            }
            continue;
        }
        if let Some(entry) = history_entry_from_command(line) {
            push_history_entry(&mut out, entry, limit);
        } else {
            dropped_invalid_entries += 1;
        }
    }
    Ok(HistoryReadReport {
        entries: out.into_iter().collect(),
        source_format: if saw_json_line { "legacy-json-lines" } else { "legacy-lines" },
        dropped_invalid_entries,
        file_bytes: 0,
    })
}

pub(crate) fn read_history_entries(path: &Path, limit: usize) -> Result<Vec<Value>> {
    Ok(read_history_report(path, limit)?.entries)
}

pub(crate) fn read_history_report(path: &Path, limit: usize) -> Result<HistoryReadReport> {
    if !path.exists() {
        return Ok(HistoryReadReport {
            entries: Vec::new(),
            source_format: "missing",
            dropped_invalid_entries: 0,
            file_bytes: 0,
        });
    }

    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_HISTORY_FILE_BYTES {
        anyhow::bail!(
            "History state exceeds {} bytes and cannot be loaded safely",
            MAX_HISTORY_FILE_BYTES
        );
    }

    let text = fs::read_to_string(path)?;
    let mut report = parse_history_entries(&text, limit)?;
    report.file_bytes = metadata.len();
    Ok(report)
}

pub(crate) fn write_history_entries(path: &Path, entries: &[Value]) -> Result<()> {
    write_json_document(path, &Value::Array(entries.to_vec()))
}

pub(crate) fn read_memory_map(path: &Path) -> Result<serde_json::Map<String, Value>> {
    if !path.exists() {
        return Ok(serde_json::Map::new());
    }
    let text = fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&text)
        .map_err(|err| anyhow::anyhow!("Malformed memory state: {err}"))?;
    let Some(object) = parsed.as_object() else {
        anyhow::bail!("Malformed memory state: expected JSON object");
    };
    Ok(object.clone())
}

pub(crate) fn write_memory_map(path: &Path, memory: &serde_json::Map<String, Value>) -> Result<()> {
    write_json_document(path, &Value::Object(memory.clone()))
}

pub(crate) fn write_json_document(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let payload = serde_json::to_string_pretty(value)?;
    atomic_write_text(path, &(payload + "\n"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_history_entries, read_history_report, MAX_HISTORY_FILE_BYTES};

    fn temp_path(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("bijux-state-store-{name}-{nanos}.json"))
    }

    #[test]
    fn parse_history_entries_rejects_json_object_payloads() {
        let error = parse_history_entries("{\"oops\":true}", 20).expect_err("must fail");
        assert!(error.to_string().contains("expected JSON array"));
    }

    #[test]
    fn parse_history_entries_rejects_malformed_json_array_payloads() {
        let error = parse_history_entries("[{\"command\":\"status\"}", 20).expect_err("must fail");
        assert!(error.to_string().contains("invalid JSON array payload"));
    }

    #[test]
    fn parse_history_entries_drops_invalid_entries_inside_arrays() {
        let report = parse_history_entries(
            "[{\"command\":\"status\"},{\"foo\":1},null,{\"command\":\"doctor\"}]",
            20,
        )
        .expect("must parse");
        assert_eq!(report.entries.len(), 2);
        assert_eq!(report.entries[0]["command"], "status");
        assert_eq!(report.entries[1]["command"], "doctor");
        assert_eq!(report.dropped_invalid_entries, 2);
    }

    #[test]
    fn parse_history_entries_limit_zero_returns_no_entries() {
        let report =
            parse_history_entries("[{\"command\":\"status\"},{\"command\":\"doctor\"}]", 0)
                .expect("must parse");
        assert!(report.entries.is_empty());
    }

    #[test]
    fn read_history_report_enforces_file_size_budget() {
        let path = temp_path("oversized");
        std::fs::write(&path, vec![b'x'; (MAX_HISTORY_FILE_BYTES + 1) as usize])
            .expect("write oversized file");
        let error = read_history_report(&path, 20).expect_err("must fail");
        assert!(error.to_string().contains("exceeds"));
        let _ = std::fs::remove_file(path);
    }
}
