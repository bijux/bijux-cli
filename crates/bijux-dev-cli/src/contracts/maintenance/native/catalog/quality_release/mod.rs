mod governance_quality;
mod release_and_plugin;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(governance_quality::rows());
    rows.extend(release_and_plugin::rows());
    rows
}
