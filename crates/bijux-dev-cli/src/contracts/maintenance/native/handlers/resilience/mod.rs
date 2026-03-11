mod evidence_surfaces;
mod hardening;
mod stress_campaigns;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    hardening::run(workspace_root, contract_id)
        .or_else(|| stress_campaigns::run(workspace_root, contract_id))
        .or_else(|| evidence_surfaces::run(workspace_root, contract_id))
}
