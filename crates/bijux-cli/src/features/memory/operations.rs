#![forbid(unsafe_code)]
//! Memory feature operations.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::infrastructure::state_store::{read_memory_map, write_memory_map};

pub(crate) fn memory_summary(memory_file: &Path) -> Result<Value> {
    let memory = read_memory_map(memory_file)?;
    Ok(json!({"status": "ok", "count": memory.len(), "message": "Memory command executed"}))
}

pub(crate) fn list_memory_keys(memory_file: &Path) -> Result<Value> {
    let memory = read_memory_map(memory_file)?;
    let mut keys: Vec<String> = memory.keys().cloned().collect();
    keys.sort_unstable();
    Ok(json!({"status": "ok", "keys": keys, "count": keys.len()}))
}

pub(crate) fn get_memory_value(memory_file: &Path, key: &str) -> Result<Value> {
    let memory = read_memory_map(memory_file)?;
    Ok(json!({"status": "ok", "key": key, "value": memory.get(key).cloned()}))
}

pub(crate) fn set_memory_pair(memory_file: &Path, pair: &str) -> Result<Value> {
    let (key, value) = pair
        .split_once('=')
        .ok_or_else(|| anyhow!("Invalid argument: expected KEY=VALUE"))?;

    let normalized_key = key.trim();
    let normalized_value = value.trim();
    let mut memory = read_memory_map(memory_file)?;
    memory.insert(
        normalized_key.to_string(),
        Value::String(normalized_value.to_string()),
    );
    write_memory_map(memory_file, &memory)?;

    Ok(json!({
        "status": "updated",
        "key": normalized_key,
        "value": normalized_value,
        "file": memory_file,
    }))
}

pub(crate) fn delete_memory_key(memory_file: &Path, key: &str) -> Result<Value> {
    let mut memory = read_memory_map(memory_file)?;
    let existed = memory.remove(key).is_some();
    write_memory_map(memory_file, &memory)?;

    Ok(json!({"status": "deleted", "key": key, "removed": existed, "file": memory_file}))
}

pub(crate) fn clear_memory(memory_file: &Path) -> Result<Value> {
    let removed = read_memory_map(memory_file)?.len();
    write_memory_map(memory_file, &serde_json::Map::new())?;

    Ok(json!({"status": "cleared", "removed_keys": removed, "file": memory_file}))
}
