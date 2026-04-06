//! Memory command handlers.

use anyhow::Result;
use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::memory::operations::{
    clear_memory, delete_memory_key, get_memory_value, list_memory_keys, memory_summary,
    set_memory_pair,
};
use crate::shared::argv::command_positionals;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "memory" => Ok(Some(memory_summary(&paths.memory_file)?)),
        [a, b] if a == "memory" && b == "list" => Ok(Some(list_memory_keys(&paths.memory_file)?)),
        [a, b] if a == "memory" && b == "get" => {
            let key = command_positionals(argv, &["memory", "get"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            Ok(Some(get_memory_value(&paths.memory_file, &key)?))
        }
        [a, b] if a == "memory" && b == "set" => {
            let raw_pair = command_positionals(argv, &["memory", "set"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY=VALUE required"))?;
            Ok(Some(set_memory_pair(&paths.memory_file, &raw_pair)?))
        }
        [a, b] if a == "memory" && b == "delete" => {
            let key = command_positionals(argv, &["memory", "delete"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: KEY required"))?;
            Ok(Some(delete_memory_key(&paths.memory_file, &key)?))
        }
        [a, b] if a == "memory" && b == "clear" => Ok(Some(clear_memory(&paths.memory_file)?)),
        _ => Ok(None),
    }
}
