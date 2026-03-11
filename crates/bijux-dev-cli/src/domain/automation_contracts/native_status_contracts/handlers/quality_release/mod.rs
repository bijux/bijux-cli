mod governance_quality;
mod release_and_plugin;

use crate::domain::automation_contracts::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    governance_quality::run(workspace_root, contract_id)
        .or_else(|| release_and_plugin::run(workspace_root, contract_id))
}
