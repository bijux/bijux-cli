//! Memory command handlers.

use anyhow::Result;
use serde_json::{json, Value};

use crate::infrastructure::state_paths::ResolvedStatePaths;
use crate::infrastructure::state_store::{read_memory_map, write_memory_map};
use crate::shared::argv::command_positionals;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "memory" => {
            let memory = read_memory_map(&paths.memory_file)?;
            Ok(Some(
                json!({"status": "ok", "count": memory.len(), "message": "Memory command executed"}),
            ))
        }
        [a, b] if a == "memory" && b == "list" => {
            let memory = read_memory_map(&paths.memory_file)?;
            let mut keys: Vec<String> = memory.keys().cloned().collect();
            keys.sort_unstable();
            Ok(Some(json!({"status": "ok", "keys": keys, "count": keys.len()})))
        }
        [a, b] if a == "memory" && b == "get" => {
            let key = command_positionals(argv, &["memory", "get"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            let memory = read_memory_map(&paths.memory_file)?;
            Ok(Some(json!({"status": "ok", "key": key, "value": memory.get(&key).cloned()})))
        }
        [a, b] if a == "memory" && b == "set" => {
            let raw_pair = command_positionals(argv, &["memory", "set"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY=VALUE required"))?;
            let (key, value) = raw_pair
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid argument: expected KEY=VALUE"))?;
            let mut memory = read_memory_map(&paths.memory_file)?;
            memory.insert(key.trim().to_string(), Value::String(value.trim().to_string()));
            write_memory_map(&paths.memory_file, &memory)?;
            Ok(Some(
                json!({"status": "updated", "key": key.trim(), "value": value.trim(), "file": paths.memory_file}),
            ))
        }
        [a, b] if a == "memory" && b == "delete" => {
            let key = command_positionals(argv, &["memory", "delete"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            let mut memory = read_memory_map(&paths.memory_file)?;
            let existed = memory.remove(&key).is_some();
            write_memory_map(&paths.memory_file, &memory)?;
            Ok(Some(
                json!({"status": "deleted", "key": key, "removed": existed, "file": paths.memory_file}),
            ))
        }
        [a, b] if a == "memory" && b == "clear" => {
            let removed = read_memory_map(&paths.memory_file)?.len();
            write_memory_map(&paths.memory_file, &serde_json::Map::new())?;
            Ok(Some(
                json!({"status": "cleared", "removed_keys": removed, "file": paths.memory_file}),
            ))
        }
        _ => Ok(None),
    }
}
