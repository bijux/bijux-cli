use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::environment_stress_campaigns_executor::run(workspace_root, contract_id)
        .or_else(|| super::command_migration_campaigns_executor::run(workspace_root, contract_id))
}
