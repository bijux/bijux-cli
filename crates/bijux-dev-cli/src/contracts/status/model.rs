//! Status contract model and identifier helpers.

use serde_json::{json, Value};

/// Stable status contract category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusContractKind {
    Generate,
    Check,
    Enforce,
    Warn,
    Run,
    Status,
}

impl StatusContractKind {
    /// Return stable lowercase string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generate => "generate",
            Self::Check => "check",
            Self::Enforce => "enforce",
            Self::Warn => "warn",
            Self::Run => "run",
            Self::Status => "status",
        }
    }

    /// Parse kind from lowercase string.
    #[must_use]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "generate" => Some(Self::Generate),
            "check" => Some(Self::Check),
            "enforce" => Some(Self::Enforce),
            "warn" => Some(Self::Warn),
            "run" => Some(Self::Run),
            "status" => Some(Self::Status),
            _ => None,
        }
    }
}

/// Infer status contract kind from stable id prefix.
#[must_use]
pub fn infer_status_contract_kind(value: &str) -> StatusContractKind {
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

/// Runtime specification for one status contract row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusContractSpec {
    pub contract_id: String,
    pub kind: StatusContractKind,
    pub source_ref: Option<String>,
    pub implementation: String,
    pub outputs: Vec<String>,
    pub command: String,
}

impl StatusContractSpec {
    /// Build spec from JSON row. Returns `None` for malformed rows.
    #[must_use]
    pub fn from_row(row: &Value) -> Option<Self> {
        let contract_id = row.get("contract_id")?.as_str()?.to_string();
        let kind = row
            .get("kind")
            .and_then(Value::as_str)
            .and_then(StatusContractKind::from_str)
            .unwrap_or_else(|| infer_status_contract_kind(&contract_id));
        let source_ref = row
            .get("source_ref")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .filter(|item| !item.is_empty());
        let implementation = row
            .get("implementation")
            .and_then(Value::as_str)
            .unwrap_or("rust")
            .to_string();
        let outputs = row
            .get("outputs")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToString::to_string))
            .collect();
        let command = row
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        Some(Self {
            contract_id,
            kind,
            source_ref,
            implementation,
            outputs,
            command,
        })
    }

    /// Convert specification back to inventory row payload.
    #[must_use]
    pub fn to_row(&self) -> Value {
        json!({
            "contract_id": self.contract_id,
            "kind": self.kind.as_str(),
            "source_ref": self.source_ref,
            "implementation": self.implementation,
            "outputs": self.outputs,
            "command": self.command,
        })
    }
}
