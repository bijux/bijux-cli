//! Status contract specification model.

use serde_json::{json, Value};

use super::id::infer_kind;
use super::kind::StatusContractKind;

/// Runtime specification for one status contract row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusContractSpec {
    pub contract_id: String,
    pub kind: StatusContractKind,
    pub source_script: Option<String>,
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
            .unwrap_or_else(|| infer_kind(&contract_id));
        let source_script = row
            .get("source_script")
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
            source_script,
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
            "source_script": self.source_script,
            "implementation": self.implementation,
            "outputs": self.outputs,
            "command": self.command,
        })
    }
}
