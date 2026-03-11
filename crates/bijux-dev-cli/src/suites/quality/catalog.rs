use crate::contract_engine::maintenance::Value;

pub(crate) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(super::state_laws_inventory::rows());
    rows.extend(super::release_evidence_inventory::rows());
    rows
}
