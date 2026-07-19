use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

fn default_container_image_reference_policy() -> bijux_dag_runtime::ContainerImageReferencePolicy {
    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CacheModeSurface {
    Off,
    Read,
    ReadWrite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MaterializeInputsSurface {
    None,
    Direct,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicySurfaceConfig {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    #[serde(default = "default_container_image_reference_policy")]
    pub container_image_reference_policy: bijux_dag_runtime::ContainerImageReferencePolicy,
    #[serde(default)]
    pub allowed_env: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSurfaceConfig {
    pub jobs: usize,
    pub cache_mode: CacheModeSurface,
    pub materialize_inputs: MaterializeInputsSurface,
    pub policy: PolicySurfaceConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PartialRuntimeSurfaceConfig {
    pub jobs: Option<usize>,
    pub cache_mode: Option<CacheModeSurface>,
    pub materialize_inputs: Option<MaterializeInputsSurface>,
    pub policy: Option<PolicySurfaceConfig>,
}

pub fn default_runtime_config() -> RuntimeSurfaceConfig {
    RuntimeSurfaceConfig {
        jobs: 1,
        cache_mode: CacheModeSurface::Off,
        materialize_inputs: MaterializeInputsSurface::None,
        policy: PolicySurfaceConfig {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
            container_image_reference_policy:
                bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
            allowed_env: Vec::new(),
        },
    }
}

pub fn resolve_effective_config(
    cli: PartialRuntimeSurfaceConfig,
    explicit: Option<PartialRuntimeSurfaceConfig>,
    env_cfg: Option<PartialRuntimeSurfaceConfig>,
    defaults: RuntimeSurfaceConfig,
) -> RuntimeSurfaceConfig {
    let mut effective = defaults;
    if let Some(env_cfg) = env_cfg {
        apply_partial(&mut effective, env_cfg);
    }
    if let Some(explicit) = explicit {
        apply_partial(&mut effective, explicit);
    }
    apply_partial(&mut effective, cli);
    normalize_runtime_config(effective)
}

pub fn normalize_runtime_config(mut cfg: RuntimeSurfaceConfig) -> RuntimeSurfaceConfig {
    let mut env_keys = BTreeSet::new();
    for key in cfg.policy.allowed_env.drain(..) {
        let normalized = key.trim().to_ascii_uppercase();
        if !normalized.is_empty() {
            env_keys.insert(normalized);
        }
    }
    cfg.policy.allowed_env = env_keys.into_iter().collect();
    cfg
}

pub fn config_fingerprint(cfg: &RuntimeSurfaceConfig) -> String {
    let normalized = normalize_runtime_config(cfg.clone());
    let bytes = serde_json::to_vec(&normalized).expect("config serialize");
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn policy_evaluation_trace(policy: &PolicySurfaceConfig) -> Vec<String> {
    let mut events = Vec::new();
    events.push(format!(
        "rule:deny_network decision:{}",
        if policy.deny_network { "deny" } else { "allow" }
    ));
    events
        .push(format!("rule:deny_env decision:{}", if policy.deny_env { "deny" } else { "allow" }));
    events.push(format!(
        "rule:deny_clock decision:{}",
        if policy.deny_clock { "deny" } else { "allow" }
    ));
    events.push(format!(
        "rule:clean_env decision:{}",
        if policy.clean_env { "enforce" } else { "skip" }
    ));
    events.push(format!(
        "rule:container_image_reference decision:{}",
        match policy.container_image_reference_policy {
            bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest => "require_digest",
            bijux_dag_runtime::ContainerImageReferencePolicy::AllowUnpinned => "allow_unpinned",
        }
    ));
    events
}

fn apply_partial(target: &mut RuntimeSurfaceConfig, partial: PartialRuntimeSurfaceConfig) {
    if let Some(jobs) = partial.jobs {
        target.jobs = jobs;
    }
    if let Some(cache_mode) = partial.cache_mode {
        target.cache_mode = cache_mode;
    }
    if let Some(materialize_inputs) = partial.materialize_inputs {
        target.materialize_inputs = materialize_inputs;
    }
    if let Some(policy) = partial.policy {
        target.policy = policy;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        config_fingerprint, default_runtime_config, normalize_runtime_config,
        policy_evaluation_trace, resolve_effective_config, CacheModeSurface,
        MaterializeInputsSurface, PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
    };

    #[test]
    fn default_runtime_config_is_stable() {
        let cfg = default_runtime_config();
        assert_eq!(cfg.jobs, 1);
        assert_eq!(cfg.cache_mode, CacheModeSurface::Off);
        assert_eq!(cfg.materialize_inputs, MaterializeInputsSurface::None);
        assert!(cfg.policy.clean_env);
    }

    #[test]
    fn normalize_runtime_config_deduplicates_and_uppercases_allowed_env() {
        let mut cfg = default_runtime_config();
        cfg.policy.allowed_env = vec![" path ".into(), "PATH".into(), "home".into()];
        let normalized = normalize_runtime_config(cfg);
        assert_eq!(normalized.policy.allowed_env, vec!["HOME", "PATH"]);
    }

    #[test]
    fn resolve_effective_config_respects_precedence() {
        let defaults = default_runtime_config();
        let env_cfg = Some(PartialRuntimeSurfaceConfig {
            jobs: Some(2),
            ..PartialRuntimeSurfaceConfig::default()
        });
        let explicit = Some(PartialRuntimeSurfaceConfig {
            jobs: Some(3),
            cache_mode: Some(CacheModeSurface::Read),
            ..PartialRuntimeSurfaceConfig::default()
        });
        let cli = PartialRuntimeSurfaceConfig {
            jobs: Some(4),
            materialize_inputs: Some(MaterializeInputsSurface::Direct),
            ..PartialRuntimeSurfaceConfig::default()
        };
        let effective = resolve_effective_config(cli, explicit, env_cfg, defaults);
        assert_eq!(effective.jobs, 4);
        assert_eq!(effective.cache_mode, CacheModeSurface::Read);
        assert_eq!(effective.materialize_inputs, MaterializeInputsSurface::Direct);
    }

    #[test]
    fn config_fingerprint_changes_on_semantic_change() {
        let mut a = default_runtime_config();
        let mut b = default_runtime_config();
        b.policy = PolicySurfaceConfig {
            deny_network: true,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
            container_image_reference_policy:
                bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
            allowed_env: vec![],
        };
        assert_ne!(config_fingerprint(&a), config_fingerprint(&b));
        a.policy.deny_network = true;
        assert_eq!(config_fingerprint(&a), config_fingerprint(&b));
    }

    #[test]
    fn policy_evaluation_trace_contains_expected_rules() {
        let policy = PolicySurfaceConfig {
            deny_network: true,
            deny_env: true,
            deny_clock: false,
            clean_env: true,
            container_image_reference_policy:
                bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
            allowed_env: vec![],
        };
        let trace = policy_evaluation_trace(&policy);
        assert!(trace.iter().any(|line| line.contains("rule:deny_network")));
        assert!(trace.iter().any(|line| line.contains("rule:deny_env")));
        assert!(trace.iter().any(|line| line.contains("rule:clean_env")));
    }
}
