#[path = "quality_governance_executor.rs"]
mod governance;
#[path = "quality_release_and_plugin_executor.rs"]
mod release_and_plugin;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    governance::run(workspace_root, contract_id)
        .or_else(|| release_and_plugin::run(workspace_root, contract_id))
}
