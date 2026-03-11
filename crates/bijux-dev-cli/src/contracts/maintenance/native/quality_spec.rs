#[path = "quality_governance_spec.rs"]
mod governance;
#[path = "quality_release_and_plugin_spec.rs"]
mod release_and_plugin;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(governance::rows());
    rows.extend(release_and_plugin::rows());
    rows
}
