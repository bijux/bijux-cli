#![forbid(unsafe_code)]
//! Compatibility helpers re-exported for Python-facing consumers.

pub use bijux_cli_install::{
    acquire_state_lock, default_compatibility_paths, discover_compatibility_paths,
    ensure_history_file, ensure_plugins_dir, load_compatibility_config, parse_compatibility_config,
    run_config_migrations, write_compatibility_config, CompatibilityConfig, CompatibilityError,
    CompatibilityPaths, PathOverrides, StateLockGuard, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
