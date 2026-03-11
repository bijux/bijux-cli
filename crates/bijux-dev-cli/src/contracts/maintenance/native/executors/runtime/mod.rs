mod config_repl_python;
mod cross_surface_consistency;
mod namespace_install;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    namespace_install::run(workspace_root, contract_id)
        .or_else(|| config_repl_python::run(workspace_root, contract_id))
        .or_else(|| cross_surface_consistency::run(workspace_root, contract_id))
}
