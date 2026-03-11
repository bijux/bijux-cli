#[path = "spec_runtime_config_repl.rs"]
mod config_repl;
#[path = "spec_runtime_cross_surface.rs"]
mod cross_surface;
#[path = "spec_runtime_namespace_install.rs"]
mod namespace_install;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(namespace_install::rows());
    rows.extend(config_repl::rows());
    rows.extend(cross_surface::rows());
    rows
}
