use crate::contract_engine::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::hardening_executor::run(workspace_root, contract_id)
        .or_else(|| super::stress_campaigns_executor::run(workspace_root, contract_id))
        .or_else(|| super::evidence_surfaces_executor::run(workspace_root, contract_id))
}
