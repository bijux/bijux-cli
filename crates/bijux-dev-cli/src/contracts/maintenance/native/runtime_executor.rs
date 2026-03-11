#[path = "runtime_config_repl_executor.rs"]
mod config_repl;
#[path = "runtime_cross_surface_executor.rs"]
mod cross_surface;
#[path = "runtime_namespace_install_executor.rs"]
mod namespace_install;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    namespace_install::run(workspace_root, contract_id)
        .or_else(|| config_repl::run(workspace_root, contract_id))
        .or_else(|| cross_surface::run(workspace_root, contract_id))
}
