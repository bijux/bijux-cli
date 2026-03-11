use crate::contract_engine::maintenance::Value;

pub(crate) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(super::namespace_install_spec::rows());
    rows.extend(super::config_repl_spec::rows());
    rows.extend(super::cross_surface_spec::rows());
    rows
}
