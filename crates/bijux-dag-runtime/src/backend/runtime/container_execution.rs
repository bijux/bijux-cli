use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerExecutionContract {
    pub image: String,
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub mounts: Vec<ContainerMount>,
    pub declared_outputs: Vec<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerMount {
    pub local_path: String,
    pub container_path: String,
    pub readonly: bool,
}

pub fn validate_container_contract(contract: &ContainerExecutionContract) -> Result<(), String> {
    if contract.image.trim().is_empty() {
        return Err("missing container image".to_string());
    }
    if contract.command.is_empty() {
        return Err("missing container command".to_string());
    }
    if contract.mounts.is_empty() {
        return Err("missing container mounts".to_string());
    }
    for output in &contract.declared_outputs {
        validate_container_relative_path(output)?;
    }
    Ok(())
}

pub fn validate_container_relative_path(path: &str) -> Result<(), String> {
    let parsed = Path::new(path);
    if parsed.is_absolute() || path.is_empty() {
        return Err(format!("invalid container relative path: {path}"));
    }
    if parsed.components().any(|c| matches!(c, Component::ParentDir | Component::Prefix(_))) {
        return Err(format!("container path escapes root: {path}"));
    }
    Ok(())
}

pub fn map_local_path_to_container(
    local_root: &Path,
    container_root: &Path,
    local_path: &Path,
) -> Result<String, String> {
    let rel = local_path
        .strip_prefix(local_root)
        .map_err(|_| format!("local path {} not under root {}", local_path.display(), local_root.display()))?;
    let mapped = container_root.join(rel);
    Ok(mapped.to_string_lossy().replace('\\', "/"))
}

pub fn container_env_isolated(
    env: &BTreeMap<String, String>,
    allowlist: &[String],
    denylist: &[String],
) -> bool {
    env.keys().all(|k| {
        let denied = denylist
            .iter()
            .any(|d| d == k || (d.ends_with('*') && k.starts_with(d.trim_end_matches('*'))));
        let allowed = allowlist.is_empty()
            || allowlist
                .iter()
                .any(|a| a == k || (a.ends_with('*') && k.starts_with(a.trim_end_matches('*'))));
        !denied && allowed
    })
}
