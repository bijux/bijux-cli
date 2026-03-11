use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::fs_process_output_integrity::run(workspace_root, contract_id)
        .or_else(|| super::parser_fuzz::run(workspace_root, contract_id))
        .or_else(|| super::fs_process_migration_notes::run(workspace_root, contract_id))
}
