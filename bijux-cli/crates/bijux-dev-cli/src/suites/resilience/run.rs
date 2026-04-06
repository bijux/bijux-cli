use crate::contracts::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::fs_process_adversarial::run(workspace_root, contract_id)
        .or_else(|| super::corruption_campaigns::run(workspace_root, contract_id))
        .or_else(|| super::fs_process_evidence_surfaces::run(workspace_root, contract_id))
}
