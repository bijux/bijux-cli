use crate::contracts::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::config_surface_reports::run(workspace_root, contract_id)
        .or_else(|| super::repl_bridge::run(workspace_root, contract_id))
        .or_else(|| super::repl_bridge_laws::run(workspace_root, contract_id))
}
