#[path = "executor_runtime_config_repl.rs"]
mod config_repl;
#[path = "executor_runtime_cross_surface.rs"]
mod cross_surface;
#[path = "executor_runtime_namespace_install.rs"]
mod namespace_install;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    namespace_install::run(workspace_root, contract_id)
        .or_else(|| config_repl::run(workspace_root, contract_id))
        .or_else(|| cross_surface::run(workspace_root, contract_id))
}
