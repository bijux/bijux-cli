mod governance;
mod release_and_plugin;

use crate::contract_engine::maintenance::Value;

pub(super) fn rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(governance::rows());
    rows.extend(release_and_plugin::rows());
    rows
}
