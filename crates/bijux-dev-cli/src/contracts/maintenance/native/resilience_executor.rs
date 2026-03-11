#[path = "resilience_evidence_surfaces_executor.rs"]
mod evidence_surfaces;
#[path = "resilience_hardening_executor.rs"]
mod hardening;
#[path = "resilience_stress_campaigns_executor.rs"]
mod stress_campaigns;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    hardening::run(workspace_root, contract_id)
        .or_else(|| stress_campaigns::run(workspace_root, contract_id))
        .or_else(|| evidence_surfaces::run(workspace_root, contract_id))
}
