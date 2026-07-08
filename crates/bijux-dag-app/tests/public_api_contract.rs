use bijux_dag_app::prelude::{
    dag_command, default_runtime_config, normalize_runtime_config, resolve_effective_config,
    RuntimeSurfaceConfig,
};
use bijux_dag_app::{CacheModeSurface, MaterializeInputsSurface, PartialRuntimeSurfaceConfig};
use clap as _;
use serde as _;
use serde_json as _;
use thiserror as _;

#[test]
fn prelude_exposes_command_embedding_entrypoints() {
    let command = dag_command();
    assert!(!command.get_name().is_empty());
}

#[test]
fn prelude_exposes_runtime_surface_configuration_flow() {
    let defaults = default_runtime_config();
    let effective = resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            jobs: Some(4),
            cache_mode: Some(CacheModeSurface::ReadWrite),
            materialize_inputs: Some(MaterializeInputsSurface::All),
            policy: None,
        },
        None,
        None,
        defaults,
    );
    let normalized: RuntimeSurfaceConfig = normalize_runtime_config(effective);
    assert_eq!(normalized.jobs, 4);
    assert!(matches!(normalized.cache_mode, CacheModeSurface::ReadWrite));
    assert!(matches!(normalized.materialize_inputs, MaterializeInputsSurface::All));
}
