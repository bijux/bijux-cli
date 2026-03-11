use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::output_integrity_executor::run(workspace_root, contract_id)
        .or_else(|| super::parser_cleanup_executor::run(workspace_root, contract_id))
        .or_else(|| super::migration_notes_executor::run(workspace_root, contract_id))
}
