use semver::{Version, VersionReq};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Plugin manifest executable contract for pre-execution validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutablePluginManifestContractV1 {
    /// Plugin namespace.
    pub namespace: String,
    /// Plugin version.
    pub version: String,
    /// Entrypoint descriptor.
    pub entrypoint: String,
    /// Declared capabilities.
    pub capabilities: Vec<String>,
    /// Trust class identifier.
    pub trust_class: String,
    /// Declared command list.
    pub commands: Vec<String>,
    /// Host compatibility range.
    pub compatibility_window: String,
}

/// Validate executable plugin manifest contract before subprocess execution.
pub fn validate_executable_plugin_manifest_contract(
    payload: &ExecutablePluginManifestContractV1,
) -> Result<(), String> {
    if payload.namespace.trim().is_empty() {
        return Err("namespace cannot be empty".to_string());
    }
    if payload.entrypoint.trim().is_empty() {
        return Err("entrypoint cannot be empty".to_string());
    }
    if payload.commands.is_empty() {
        return Err("commands cannot be empty".to_string());
    }
    if payload.capabilities.is_empty() {
        return Err("capabilities cannot be empty".to_string());
    }
    if payload.trust_class.trim().is_empty() {
        return Err("trust_class cannot be empty".to_string());
    }
    Version::parse(&payload.version).map_err(|error| format!("invalid version: {error}"))?;
    VersionReq::parse(&payload.compatibility_window)
        .map_err(|error| format!("invalid compatibility_window: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_executable_plugin_manifest_contract, ExecutablePluginManifestContractV1,
    };

    #[test]
    fn g011_plugin_manifest_contract_refuses_invalid_compatibility_window() {
        let manifest = ExecutablePluginManifestContractV1 {
            namespace: "community-tools".to_string(),
            version: "1.2.0".to_string(),
            entrypoint: "plugin:main".to_string(),
            capabilities: vec!["inspect".to_string(), "validate".to_string()],
            trust_class: "local".to_string(),
            commands: vec!["community lint".to_string()],
            compatibility_window: "not-semver".to_string(),
        };
        assert!(validate_executable_plugin_manifest_contract(&manifest).is_err());
    }
}
