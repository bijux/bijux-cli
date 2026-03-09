#![forbid(unsafe_code)]
//! Output encoding and envelope rendering surfaces.

use bijux_cli_contracts::ContractMarker;

/// Render a marker as compact JSON.
pub fn to_json(marker: &ContractMarker) -> Result<String, serde_json::Error> {
    serde_json::to_string(marker)
}

/// Render a marker as YAML.
pub fn to_yaml(marker: &ContractMarker) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(marker)
}
