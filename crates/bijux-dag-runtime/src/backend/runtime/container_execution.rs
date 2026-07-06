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
    #[serde(default)]
    pub gpu_devices: u32,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerMount {
    pub local_path: String,
    pub container_path: String,
    pub readonly: bool,
}

pub fn supported_container_engines() -> &'static [&'static str] {
    &["docker", "podman"]
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

pub fn container_engine_discovery(engine: &str) -> Result<String, String> {
    let output = std::process::Command::new(engine)
        .arg("--version")
        .output()
        .map_err(|_| format!("container engine unavailable: {}", engine))?;
    if !output.status.success() {
        return Err(format!("container engine unavailable: {}", engine));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err(format!("container engine unavailable: {}", engine));
    }
    Ok(version)
}

pub fn container_network_policy_args(
    engine: &str,
    deny_network: bool,
) -> Result<Vec<String>, String> {
    if !deny_network {
        return Ok(Vec::new());
    }
    if supported_container_engines().iter().any(|candidate| candidate == &engine) {
        return Ok(vec!["--network".to_string(), "none".to_string()]);
    }
    Err(format!(
        "container engine {} cannot enforce deny_network with the built-in adapter",
        engine
    ))
}

pub fn container_gpu_runtime_args(engine: &str, gpu_devices: u32) -> Result<Vec<String>, String> {
    if gpu_devices == 0 {
        return Ok(Vec::new());
    }
    if supported_container_engines().iter().any(|candidate| candidate == &engine) {
        return Ok(vec![format!("--gpus={gpu_devices}")]);
    }
    Err(format!("container engine {} cannot request gpu devices with the built-in adapter", engine))
}

pub fn container_volume_contract(node_dir: &Path) -> Vec<ContainerMount> {
    vec![
        ContainerMount {
            local_path: node_dir.join("inputs").display().to_string(),
            container_path: "/bijux/node/inputs".to_string(),
            readonly: true,
        },
        ContainerMount {
            local_path: node_dir.join("outputs").display().to_string(),
            container_path: "/bijux/node/outputs".to_string(),
            readonly: false,
        },
        ContainerMount {
            local_path: node_dir.join("work").display().to_string(),
            container_path: "/bijux/node/work".to_string(),
            readonly: false,
        },
    ]
}

pub fn validate_container_mount_contract(
    mounts: &[ContainerMount],
    node_dir: &Path,
) -> Result<(), String> {
    if mounts.is_empty() {
        return Err("missing container mounts".to_string());
    }
    let allowed_root = node_dir.to_string_lossy().replace('\\', "/");
    let mut inputs_ok = false;
    let mut outputs_ok = false;
    let mut work_ok = false;
    for mount in mounts {
        let normalized_local = mount.local_path.replace('\\', "/");
        if !normalized_local.starts_with(&allowed_root) {
            return Err(format!("container mount escapes node root: {}", mount.local_path));
        }
        match mount.container_path.as_str() {
            "/bijux/node/inputs" => {
                inputs_ok = mount.readonly;
            }
            "/bijux/node/outputs" => {
                outputs_ok = !mount.readonly;
            }
            "/bijux/node/work" => {
                work_ok = !mount.readonly;
            }
            _ => {
                return Err(format!("unexpected container mount target: {}", mount.container_path))
            }
        }
    }
    if !inputs_ok {
        return Err("container inputs mount must be read-only".to_string());
    }
    if !outputs_ok {
        return Err("container outputs mount must be writable".to_string());
    }
    if !work_ok {
        return Err("container work mount must be writable".to_string());
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
    let rel = local_path.strip_prefix(local_root).map_err(|_| {
        format!("local path {} not under root {}", local_path.display(), local_root.display())
    })?;
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
