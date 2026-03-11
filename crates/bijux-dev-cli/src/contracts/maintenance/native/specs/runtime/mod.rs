mod config_repl;
mod cross_surface;
mod namespace_install;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(namespace_install::rows());
    rows.extend(config_repl::rows());
    rows.extend(cross_surface::rows());
    rows
}
