use crate::contract_engine::maintenance::Value;

pub(crate) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(super::install_runtime_inventory::rows());
    rows.extend(super::config_surface_inventory::rows());
    rows.extend(super::cross_surface_inventory::rows());
    rows
}
