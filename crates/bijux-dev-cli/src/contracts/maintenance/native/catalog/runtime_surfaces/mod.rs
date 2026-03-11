mod config_repl_python;
mod cross_surface_consistency;
mod namespace_install;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(namespace_install::rows());
    rows.extend(config_repl_python::rows());
    rows.extend(cross_surface_consistency::rows());
    rows
}
