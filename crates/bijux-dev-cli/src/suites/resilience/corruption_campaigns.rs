use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::fs_process_environment_stress::run(workspace_root, contract_id)
        .or_else(|| super::corruption_campaigns_command_migration::run(workspace_root, contract_id))
}
