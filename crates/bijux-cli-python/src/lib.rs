#![forbid(unsafe_code)]
//! Python compatibility bridge surfaces.

mod bindings;
mod compatibility;
mod conversions;

pub use bindings::{
    cli_status_binding_api, command_tree_introspection_api, config_resolution_api,
    doctor_binding_api, execution_facade_api, execution_outcome_api, install_path_helpers_api,
    plugin_registry_inspection_api, plugins_list_binding_api, python_bridge_marker,
    repl_bootstrap_binding_api, schema_export_helpers_api, status_binding_api, version_binding_api,
};
pub use compatibility::{
    acquire_state_lock, default_compatibility_paths, discover_compatibility_paths,
    ensure_history_file, ensure_plugins_dir, load_compatibility_config, parse_compatibility_config,
    run_config_migrations, write_compatibility_config, CompatibilityConfig, CompatibilityError,
    CompatibilityPaths, PathOverrides, StateLockGuard, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
pub use conversions::{
    classify_core_error, classify_failure, python_exception_tag, BridgeErrorKind,
};

#[cfg(feature = "python-extension")]
mod python_extension {
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    use crate::{
        command_tree_introspection_api, execution_facade_api, install_path_helpers_api,
        plugin_registry_inspection_api,
    };

    #[pyfunction]
    fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    #[pyfunction]
    fn command_tree_introspection() -> String {
        command_tree_introspection_api()
    }

    #[pyfunction]
    fn execution_facade(args: Vec<String>) -> PyResult<String> {
        execution_facade_api(&args).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    #[pyfunction]
    fn install_paths(home_dir: String) -> String {
        install_path_helpers_api(std::path::Path::new(&home_dir))
    }

    #[pyfunction]
    fn plugin_registry_inspection(registry_path: String) -> PyResult<String> {
        plugin_registry_inspection_api(std::path::Path::new(&registry_path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    #[pymodule]
    fn _native(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add_function(wrap_pyfunction!(version, module)?)?;
        module.add_function(wrap_pyfunction!(command_tree_introspection, module)?)?;
        module.add_function(wrap_pyfunction!(execution_facade, module)?)?;
        module.add_function(wrap_pyfunction!(install_paths, module)?)?;
        module.add_function(wrap_pyfunction!(plugin_registry_inspection, module)?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::{
        discover_compatibility_paths, parse_compatibility_config, CompatibilityConfig,
        CompatibilityError, PathOverrides, ENV_HISTORY_PATH,
    };

    #[test]
    fn applies_precedence_and_path_normalization() {
        let home = PathBuf::from("/tmp/home");
        let mut env_map = HashMap::new();
        env_map.insert(ENV_HISTORY_PATH.to_string(), "~/history.log".to_string());

        let config = CompatibilityConfig {
            config_file: Some(PathBuf::from("config/custom.env")),
            history_file: None,
            plugins_dir: Some(PathBuf::from("plugins")),
        };

        let overrides = PathOverrides {
            config_file: Some(PathBuf::from("/custom/config.env")),
            history_file: None,
            plugins_dir: None,
        };

        let resolved =
            discover_compatibility_paths(Some(&home), &overrides, &env_map, &config).expect("ok");

        assert_eq!(resolved.config_file, PathBuf::from("/custom/config.env"));
        assert_eq!(
            resolved.history_file,
            PathBuf::from("/tmp/home/history.log")
        );
        assert_eq!(resolved.plugins_dir, PathBuf::from("/tmp/home/plugins"));
    }

    #[test]
    fn parses_known_keys_and_rejects_unknown_keys() {
        let parsed = parse_compatibility_config(
            "BIJUXCLI_CONFIG=~/cfg.env\nBIJUXCLI_HISTORY_FILE=~/h.log\n",
        )
        .expect("should parse");
        assert_eq!(parsed.config_file, Some(PathBuf::from("~/cfg.env")));
        assert_eq!(parsed.history_file, Some(PathBuf::from("~/h.log")));

        let unknown = parse_compatibility_config("RANDOM_KEY=1\n").expect_err("must fail");
        assert!(matches!(
            unknown,
            CompatibilityError::UnsupportedConfigKey(_)
        ));
    }
}
