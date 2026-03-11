#![forbid(unsafe_code)]
//! Installation, compatibility, and runtime identity facade.

pub use crate::features::install::{
    acquire_state_lock, atomic_write_text, canonical_crate_name, cargo_install_strategy,
    default_compatibility_paths, detect_stale_wrapper_scripts, discover_compatibility_paths,
    discover_path_binaries, ensure_history_file, ensure_plugins_dir, initialize_first_run_state,
    install_health_report, legacy_installer_conflicts, load_compatibility_config,
    parse_compatibility_config, pip_install_strategy, post_install_hint, query,
    resolve_active_binary, run_config_migrations, write_compatibility_config, CompatibilityConfig,
    CompatibilityError, CompatibilityPaths, CompletionShell, Ecosystem, InstallHealthReport,
    InstallStrategy, PackageChannel, PathOverrides, StateLockGuard, CANONICAL_EXECUTABLE,
    ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
};
