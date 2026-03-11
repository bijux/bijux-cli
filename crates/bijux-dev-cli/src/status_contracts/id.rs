//! Stable status contract identifier helpers.

use super::kind::StatusContractKind;

/// Stable status contract id prefix.
pub const STATUS_CONTRACT_PREFIX: &str = "STATUS-CONTRACT-";

/// Return `true` when a value matches the status contract id namespace.
#[must_use]
pub fn is_status_contract_id(value: &str) -> bool {
    value.starts_with(STATUS_CONTRACT_PREFIX)
}

/// Infer status contract kind from stable id prefix.
#[must_use]
pub fn infer_kind(value: &str) -> StatusContractKind {
    if value.starts_with("STATUS-CONTRACT-GENERATE-") {
        StatusContractKind::Generate
    } else if value.starts_with("STATUS-CONTRACT-CHECK-") {
        StatusContractKind::Check
    } else if value.starts_with("STATUS-CONTRACT-ENFORCE-") {
        StatusContractKind::Enforce
    } else if value.starts_with("STATUS-CONTRACT-WARN-") {
        StatusContractKind::Warn
    } else if value.starts_with("STATUS-CONTRACT-RUN-") {
        StatusContractKind::Run
    } else {
        StatusContractKind::Status
    }
}
