#[path = "spec_quality_governance.rs"]
mod governance;
#[path = "spec_quality_release_and_plugin.rs"]
mod release_and_plugin;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(governance::rows());
    rows.extend(release_and_plugin::rows());
    rows
}
