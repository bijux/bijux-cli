#![forbid(unsafe_code)]
//! Installation, compatibility, and runtime identity facade.

pub use crate::features::install::{
    acquire_state_lock, canonical_crate_name, cargo_install_strategy, default_compatibility_paths,
    discover_compatibility_paths, ensure_history_file, ensure_plugins_dir, install_health_report,
    install_target_aliases, load_compatibility_config, parse_compatibility_config,
    pip_install_strategy, query, resolve_install_target, run_config_migrations,
    write_compatibility_config, CompatibilityConfig, CompatibilityError, CompatibilityPaths,
    InstallTarget, PathOverrides, StateLockGuard, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
