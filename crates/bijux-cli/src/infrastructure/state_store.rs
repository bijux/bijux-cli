//! State-file persistence helpers shared by command handlers.

use std::fs;
use std::path::Path;

use anyhow::Result;
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

fn parse_history_entries(text: &str) -> Result<Vec<Value>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        if let Some(items) = value.as_array() {
            return Ok(items.iter().filter(|item| item.is_object()).cloned().collect());
        }
        anyhow::bail!("Unexpected history file format (not JSON array)");
    }

    // Compatibility fallback for line-oriented history files with partial corruption.
    let mut out = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            if value.is_object() {
                out.push(value);
            }
            continue;
        }
        out.push(history_entry_from_command(line));
    }
    Ok(out)
}

pub(crate) fn read_history_entries(path: &Path, limit: usize) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    let mut entries = parse_history_entries(&text)?;
    if entries.len() > limit {
        entries = entries.split_off(entries.len() - limit);
    }
    Ok(entries)
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
