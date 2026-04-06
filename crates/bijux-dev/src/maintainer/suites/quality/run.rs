use crate::contracts::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::state_laws_governance::run(workspace_root, contract_id)
        .or_else(|| super::release_evidence::run(workspace_root, contract_id))
}
