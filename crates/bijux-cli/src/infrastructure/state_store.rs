//! State-file persistence helpers shared by command handlers.

use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde_json::{json, Value};

use crate::infrastructure::fs_store::atomic_write_text;

fn history_entry_from_command(command: &str) -> Value {
    json!({
        "command": command,
        "params": [],
        "timestamp": 0.0,
        "success": true,
        "return_code": 0,
        "duration_ms": 0.0,
        "raw": {},
    })
}

fn push_history_entry(entries: &mut VecDeque<Value>, entry: Value, limit: usize) {
    if limit != usize::MAX && entries.len() == limit {
        entries.pop_front();
    }
    entries.push_back(entry);
}

struct LimitedHistoryArraySeed {
    limit: usize,
}

impl<'de> DeserializeSeed<'de> for LimitedHistoryArraySeed {
    type Value = Vec<Value>;

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

impl<'de> Visitor<'de> for LimitedHistoryArrayVisitor {
    type Value = Vec<Value>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON array of history entries")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut entries = VecDeque::<Value>::new();
        while let Some(value) = seq.next_element::<Value>()? {
            if value.is_object() {
                push_history_entry(&mut entries, value, self.limit);
            }
        }
        Ok(entries.into_iter().collect())
    }
}

fn parse_history_array_entries(text: &str, limit: usize) -> Result<Vec<Value>> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let entries = LimitedHistoryArraySeed { limit }
        .deserialize(&mut deserializer)
        .map_err(|err| anyhow::anyhow!(err))?;
    deserializer.end().map_err(|err| anyhow::anyhow!(err))?;
    Ok(entries)
}

fn parse_history_entries(text: &str, limit: usize) -> Result<Vec<Value>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        if let Ok(entries) = parse_history_array_entries(trimmed, limit) {
            return Ok(entries);
        }
    }

    if serde_json::from_str::<Value>(trimmed).is_ok() {
        anyhow::bail!("Unexpected history file format (not JSON array)");
    }

    // Compatibility fallback for line-oriented history files with partial corruption.
    let mut out = VecDeque::<Value>::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            if value.is_object() {
                push_history_entry(&mut out, value, limit);
            }
            continue;
        }
        push_history_entry(&mut out, history_entry_from_command(line), limit);
    }
    Ok(out.into_iter().collect())
}

pub(crate) fn read_history_entries(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    parse_history_entries(&text, limit)
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
