use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stable result marker used by integration boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContractMarker {
    /// Contract namespace identifier.
    pub namespace: String,
}
