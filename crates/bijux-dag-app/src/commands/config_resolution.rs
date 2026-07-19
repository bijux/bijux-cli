use crate::commands::{CacheModeArg, MaterializeModeArg};
use crate::config_surface::{
    config_fingerprint, default_runtime_config, policy_evaluation_trace, resolve_effective_config,
    CacheModeSurface, MaterializeInputsSurface, PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
    RuntimeSurfaceConfig,
};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub(crate) struct ShowEffectiveConfigRequest<'a> {
    pub config_path: Option<&'a Path>,
    pub jobs: Option<usize>,
    pub cache_mode: Option<CacheModeArg>,
    pub materialize_inputs: Option<MaterializeModeArg>,
}

pub(crate) struct ShowEffectivePolicyRequest<'a> {
    pub config_path: Option<&'a Path>,
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    pub allow_env: &'a [String],
}

pub(crate) fn show_effective_config(
    req: ShowEffectiveConfigRequest<'_>,
) -> Result<RuntimeSurfaceConfig, ExitCode> {
    let explicit = load_partial_config(req.config_path)?;
    let env_cfg = env_partial_runtime_config();
    Ok(resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            jobs: req.jobs,
            cache_mode: req.cache_mode.map(map_cache_mode_surface),
            materialize_inputs: req.materialize_inputs.map(map_materialize_surface),
            policy: None,
        },
        explicit,
        env_cfg,
        default_runtime_config(),
    ))
}

pub(crate) fn show_effective_policy(
    req: ShowEffectivePolicyRequest<'_>,
) -> Result<Value, ExitCode> {
    let explicit = load_partial_config(req.config_path)?;
    let env_cfg = env_partial_runtime_config();
    let cli_policy = if req.deny_network
        || req.deny_env
        || req.deny_clock
        || req.clean_env
        || !req.allow_env.is_empty()
    {
        Some(PolicySurfaceConfig {
            deny_network: req.deny_network,
            deny_env: req.deny_env,
            deny_clock: req.deny_clock,
            clean_env: req.clean_env,
            container_image_reference_policy:
                bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
            allowed_env: req.allow_env.to_vec(),
        })
    } else {
        None
    };
    let effective = resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            policy: cli_policy,
            ..PartialRuntimeSurfaceConfig::default()
        },
        explicit,
        env_cfg,
        default_runtime_config(),
    );
    Ok(json!({
        "effective_policy": effective.policy,
        "trace": policy_evaluation_trace(&effective.policy),
        "config_fingerprint": config_fingerprint(&effective),
    }))
}

fn env_partial_runtime_config() -> Option<PartialRuntimeSurfaceConfig> {
    let mut partial = PartialRuntimeSurfaceConfig::default();
    if let Ok(raw_jobs) = std::env::var("BIJUX_DAG_JOBS") {
        if let Ok(jobs) = raw_jobs.parse::<usize>() {
            partial.jobs = Some(jobs);
        }
    }
    if let Ok(raw_cache_mode) = std::env::var("BIJUX_DAG_CACHE_MODE") {
        partial.cache_mode = match raw_cache_mode.to_ascii_lowercase().as_str() {
            "off" => Some(CacheModeSurface::Off),
            "read" => Some(CacheModeSurface::Read),
            "read-write" | "readwrite" => Some(CacheModeSurface::ReadWrite),
            _ => None,
        };
    }
    if let Ok(raw_materialize) = std::env::var("BIJUX_DAG_MATERIALIZE_INPUTS") {
        partial.materialize_inputs = match raw_materialize.to_ascii_lowercase().as_str() {
            "none" => Some(MaterializeInputsSurface::None),
            "direct" => Some(MaterializeInputsSurface::Direct),
            "all" => Some(MaterializeInputsSurface::All),
            _ => None,
        };
    }
    if let Ok(raw_policy) = std::env::var("BIJUX_DAG_POLICY_JSON") {
        if let Ok(policy) = serde_json::from_str::<PolicySurfaceConfig>(&raw_policy) {
            partial.policy = Some(policy);
        }
    }
    if partial == PartialRuntimeSurfaceConfig::default() {
        None
    } else {
        Some(partial)
    }
}

fn load_partial_config(
    path: Option<&Path>,
) -> Result<Option<PartialRuntimeSurfaceConfig>, ExitCode> {
    let Some(path) = path else {
        return Ok(None);
    };
    let payload = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    let parsed = serde_json::from_str::<PartialRuntimeSurfaceConfig>(&payload)
        .map_err(|_| ExitCode::from(2))?;
    Ok(Some(parsed))
}

fn map_cache_mode_surface(mode: CacheModeArg) -> CacheModeSurface {
    match mode {
        CacheModeArg::Off => CacheModeSurface::Off,
        CacheModeArg::Read => CacheModeSurface::Read,
        CacheModeArg::Readwrite => CacheModeSurface::ReadWrite,
    }
}

fn map_materialize_surface(mode: MaterializeModeArg) -> MaterializeInputsSurface {
    match mode {
        MaterializeModeArg::Copy => MaterializeInputsSurface::All,
        MaterializeModeArg::Hardlink => MaterializeInputsSurface::Direct,
        MaterializeModeArg::Symlink => MaterializeInputsSurface::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_cache_mode_surface, map_materialize_surface, show_effective_config,
        show_effective_policy, ShowEffectiveConfigRequest, ShowEffectivePolicyRequest,
    };
    use crate::commands::{CacheModeArg, MaterializeModeArg};
    use crate::config_surface::{CacheModeSurface, MaterializeInputsSurface};
    use std::process::ExitCode;

    #[test]
    fn mapping_functions_translate_cli_modes() {
        assert_eq!(map_cache_mode_surface(CacheModeArg::Readwrite), CacheModeSurface::ReadWrite);
        assert_eq!(
            map_materialize_surface(MaterializeModeArg::Hardlink),
            MaterializeInputsSurface::Direct
        );
    }

    #[test]
    fn show_effective_config_prefers_cli_over_defaults() {
        let cfg = show_effective_config(ShowEffectiveConfigRequest {
            config_path: None,
            jobs: Some(4),
            cache_mode: Some(CacheModeArg::Read),
            materialize_inputs: Some(MaterializeModeArg::Copy),
        })
        .expect("resolve config");
        assert_eq!(cfg.jobs, 4);
        assert_eq!(cfg.cache_mode, CacheModeSurface::Read);
        assert_eq!(cfg.materialize_inputs, MaterializeInputsSurface::All);
    }

    #[test]
    fn show_effective_policy_emits_trace_and_fingerprint() {
        let payload = show_effective_policy(ShowEffectivePolicyRequest {
            config_path: None,
            deny_network: true,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
            allow_env: &["path".to_string(), "PATH".to_string()],
        })
        .expect("policy payload");
        assert!(payload.get("trace").and_then(|v| v.as_array()).is_some());
        assert!(payload.get("config_fingerprint").and_then(|v| v.as_str()).is_some());
    }

    #[test]
    fn show_effective_config_rejects_invalid_config_json() {
        let tmp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(tmp.path(), "{not-json}").expect("write invalid json");
        let err = show_effective_config(ShowEffectiveConfigRequest {
            config_path: Some(tmp.path()),
            jobs: None,
            cache_mode: None,
            materialize_inputs: None,
        })
        .expect_err("invalid config should fail");
        assert_eq!(err, ExitCode::from(2));
    }
}
