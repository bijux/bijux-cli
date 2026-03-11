#[path = "executor_quality_governance.rs"]
mod governance;
#[path = "executor_quality_release_and_plugin.rs"]
mod release_and_plugin;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    governance::run(workspace_root, contract_id)
        .or_else(|| release_and_plugin::run(workspace_root, contract_id))
}
