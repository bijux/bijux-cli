use crate::contract_engine::maintenance::Value;

pub(crate) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(super::governance_spec::rows());
    rows.extend(super::release_and_plugin_spec::rows());
    rows
}
