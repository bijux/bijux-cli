#[path = "runtime_config_repl_spec.rs"]
mod config_repl;
#[path = "runtime_cross_surface_spec.rs"]
mod cross_surface;
#[path = "runtime_namespace_install_spec.rs"]
mod namespace_install;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(namespace_install::rows());
    rows.extend(config_repl::rows());
    rows.extend(cross_surface::rows());
    rows
}
