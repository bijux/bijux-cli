#![forbid(unsafe_code)]
//! Python compatibility bridge surfaces.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli_contracts::ContractMarker;
use thiserror::Error;

/// Environment variable used for explicit config file path.
pub const ENV_CONFIG_PATH: &str = "BIJUXCLI_CONFIG";
/// Environment variable used for explicit history file path.
pub const ENV_HISTORY_PATH: &str = "BIJUXCLI_HISTORY_FILE";
/// Environment variable used for explicit plugin directory path.
pub const ENV_PLUGINS_PATH: &str = "BIJUXCLI_PLUGINS_DIR";

/// Compatibility paths consumed by Python and Rust implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPaths {
    /// Path to `config.env`.
    pub config_file: PathBuf,
    /// Path to history store.
    pub history_file: PathBuf,
    /// Path to plugins directory.
    pub plugins_dir: PathBuf,
}

/// Key-based path overrides from command-line flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathOverrides {
    /// Optional override for config file path.
    pub config_file: Option<PathBuf>,
    /// Optional override for history file path.
    pub history_file: Option<PathBuf>,
    /// Optional override for plugins directory path.
    pub plugins_dir: Option<PathBuf>,
}

/// Parsed file-backed compatibility configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompatibilityConfig {
    /// Optional path from config file for config path recursion-safe representation.
    pub config_file: Option<PathBuf>,
    /// Optional path from config file for history file.
    pub history_file: Option<PathBuf>,
    /// Optional path from config file for plugins directory.
    pub plugins_dir: Option<PathBuf>,
}

/// Error type for compatibility discovery and file operations.
#[derive(Debug, Error)]
pub enum CompatibilityError {
    /// Home directory not provided.
    #[error("home directory is required for compatibility path discovery")]
    MissingHome,
    /// Config file contained an unknown key.
    #[error("unsupported config key: {0}")]
    UnsupportedConfigKey(String),
    /// Config file contains malformed line.
    #[error("malformed config line {line}: {content}")]
    MalformedConfigLine {
        /// 1-based line number.
        line: usize,
        /// Original line content.
        content: String,
    },
    /// Lock file already exists for mutable state operation.
    #[error("state lock is already held at {0}")]
    LockHeld(PathBuf),
    /// Underlying I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Build python-bridge marker.
#[must_use]
pub fn python_bridge_marker() -> ContractMarker {
    ContractMarker { namespace: "python-bridge".to_string() }
}

/// Resolve effective compatibility paths with strict precedence:
/// CLI flag overrides -> environment variables -> config file -> defaults.
pub fn discover_compatibility_paths(
    home_dir: Option<&Path>,
    cli_overrides: &PathOverrides,
    env_map: &HashMap<String, String>,
    file_config: &CompatibilityConfig,
) -> Result<CompatibilityPaths, CompatibilityError> {
    let home = home_dir.ok_or(CompatibilityError::MissingHome)?;
    let defaults = default_compatibility_paths(home);

    let config_file = select_path(
        cli_overrides.config_file.as_ref(),
        env_map.get(ENV_CONFIG_PATH),
        file_config.config_file.as_ref(),
        &defaults.config_file,
        home,
    );
    let history_file = select_path(
        cli_overrides.history_file.as_ref(),
        env_map.get(ENV_HISTORY_PATH),
        file_config.history_file.as_ref(),
        &defaults.history_file,
        home,
    );
    let plugins_dir = select_path(
        cli_overrides.plugins_dir.as_ref(),
        env_map.get(ENV_PLUGINS_PATH),
        file_config.plugins_dir.as_ref(),
        &defaults.plugins_dir,
        home,
    );

    Ok(CompatibilityPaths { config_file, history_file, plugins_dir })
}

/// Default compatibility paths anchored in the user home directory.
#[must_use]
pub fn default_compatibility_paths(home_dir: &Path) -> CompatibilityPaths {
    let base = home_dir.join(".bijux");
    CompatibilityPaths {
        config_file: base.join(".env"),
        history_file: base.join(".history"),
        plugins_dir: base.join(".plugins"),
    }
}

/// Parse `.env`-style configuration file.
pub fn parse_compatibility_config(text: &str) -> Result<CompatibilityConfig, CompatibilityError> {
    let mut values = BTreeMap::<String, String>::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(CompatibilityError::MalformedConfigLine {
                line: line_no,
                content: raw_line.to_string(),
            });
        };

        let trimmed_key = key.trim();
        let trimmed_value = value.trim();
        match trimmed_key {
            ENV_CONFIG_PATH | ENV_HISTORY_PATH | ENV_PLUGINS_PATH => {
                values.insert(trimmed_key.to_string(), trimmed_value.to_string());
            }
            _ => {
                return Err(CompatibilityError::UnsupportedConfigKey(trimmed_key.to_string()));
            }
        }
    }

    Ok(CompatibilityConfig {
        config_file: values.get(ENV_CONFIG_PATH).map(PathBuf::from),
        history_file: values.get(ENV_HISTORY_PATH).map(PathBuf::from),
        plugins_dir: values.get(ENV_PLUGINS_PATH).map(PathBuf::from),
    })
}

/// Read and parse compatibility config file if it exists.
pub fn load_compatibility_config(path: &Path) -> Result<CompatibilityConfig, CompatibilityError> {
    if !path.exists() {
        return Ok(CompatibilityConfig::default());
    }

    let text = fs::read_to_string(path)?;
    parse_compatibility_config(&text)
}

/// Persist compatibility config atomically.
pub fn write_compatibility_config(
    path: &Path,
    config: &CompatibilityConfig,
) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut lines = Vec::new();
    if let Some(value) = &config.config_file {
        lines.push(format!("{ENV_CONFIG_PATH}={}", value.display()));
    }
    if let Some(value) = &config.history_file {
        lines.push(format!("{ENV_HISTORY_PATH}={}", value.display()));
    }
    if let Some(value) = &config.plugins_dir {
        lines.push(format!("{ENV_PLUGINS_PATH}={}", value.display()));
    }
    lines.sort();

    let rendered = if lines.is_empty() {
        String::new()
    } else {
        let mut buf = lines.join("\n");
        buf.push('\n');
        buf
    };

    let temp_path = path.with_extension("tmp");
    {
        let mut temp = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;
        temp.write_all(rendered.as_bytes())?;
        temp.sync_all()?;
    }

    fs::rename(temp_path, path)?;
    Ok(())
}

/// Acquire process lock for mutable state operations.
pub fn acquire_state_lock(lock_path: &Path) -> Result<StateLockGuard, CompatibilityError> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    match OpenOptions::new().create_new(true).write(true).open(lock_path) {
        Ok(mut file) => {
            file.write_all(b"bijux-cli lock\n")?;
            file.sync_all()?;
            Ok(StateLockGuard { path: lock_path.to_path_buf() })
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(CompatibilityError::LockHeld(lock_path.to_path_buf()))
        }
        Err(error) => Err(CompatibilityError::Io(error)),
    }
}

/// Guard that removes the lock path when dropped.
#[derive(Debug)]
pub struct StateLockGuard {
    path: PathBuf,
}

impl Drop for StateLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Ensure history file exists and parent directory is present.
pub fn ensure_history_file(path: &Path) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
        file.write_all(b"[]\n")?;
        file.sync_all()?;
    }

    Ok(())
}

/// Ensure plugin directory exists.
pub fn ensure_plugins_dir(path: &Path) -> Result<(), CompatibilityError> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Placeholder migration entrypoint for forward config evolution.
pub fn run_config_migrations(_config_path: &Path, _current_version: u32) -> Result<(), CompatibilityError> {
    Ok(())
}

/// Return Rust-backed version string for Python bindings.
#[must_use]
pub fn version_api() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Return command tree introspection payload as JSON.
#[must_use]
pub fn command_tree_introspection_api() -> String {
    serde_json::json!({
        "root": "bijux",
        "namespaces": ["cli", "dev", "help", "version", "doctor", "repl", "plugins", "completion", "inspect"],
    })
    .to_string()
}

/// Execute the Rust-backed CLI facade through the canonical runtime binary.
pub fn execution_facade_api(argv: &[String]) -> Result<String, CompatibilityError> {
    let binary = env::var("BIJUX_BIN").unwrap_or_else(|_| "bijux-rs".to_string());
    let output = Command::new(binary).args(argv).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Resolve compatibility paths and return JSON payload for Python consumers.
pub fn config_resolution_api(
    home_dir: Option<&Path>,
    cli_overrides: &PathOverrides,
    env_map: &HashMap<String, String>,
    file_config: &CompatibilityConfig,
) -> Result<String, CompatibilityError> {
    let resolved = discover_compatibility_paths(home_dir, cli_overrides, env_map, file_config)?;
    Ok(serde_json::json!({
        "config_file": resolved.config_file,
        "history_file": resolved.history_file,
        "plugins_dir": resolved.plugins_dir,
    })
    .to_string())
}

/// Return install-path helpers as JSON.
#[must_use]
pub fn install_path_helpers_api(home_dir: &Path) -> String {
    let defaults = default_compatibility_paths(home_dir);
    serde_json::json!({
        "config_file": defaults.config_file,
        "history_file": defaults.history_file,
        "plugins_dir": defaults.plugins_dir,
    })
    .to_string()
}

/// Return plugin registry inspection payload as JSON.
pub fn plugin_registry_inspection_api(registry_path: &Path) -> Result<String, CompatibilityError> {
    if !registry_path.exists() {
        return Ok("{\"version\":\"1\",\"plugins\":{}}".to_string());
    }
    let text = fs::read_to_string(registry_path)?;
    Ok(text)
}

fn select_path(
    cli_value: Option<&PathBuf>,
    env_value: Option<&String>,
    config_value: Option<&PathBuf>,
    default_value: &Path,
    home_dir: &Path,
) -> PathBuf {
    let candidate = cli_value
        .cloned()
        .or_else(|| env_value.map(PathBuf::from))
        .or_else(|| config_value.cloned())
        .unwrap_or_else(|| default_value.to_path_buf());

    normalize_path(&candidate, home_dir)
}

fn normalize_path(path: &Path, home_dir: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };

    if raw == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(tail) = raw.strip_prefix("~/") {
        return home_dir.join(tail);
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }

    home_dir.join(path)
}

#[cfg(feature = "python-extension")]
mod python_extension {
    use super::*;
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::prelude::*;

    #[pyfunction]
    fn version() -> String {
        version_api()
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
        install_path_helpers_api(Path::new(&home_dir))
    }

    #[pyfunction]
    fn plugin_registry_inspection(registry_path: String) -> PyResult<String> {
        plugin_registry_inspection_api(Path::new(&registry_path))
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
    use super::*;

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
        assert_eq!(resolved.history_file, PathBuf::from("/tmp/home/history.log"));
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
        assert!(matches!(unknown, CompatibilityError::UnsupportedConfigKey(_)));
    }
}
