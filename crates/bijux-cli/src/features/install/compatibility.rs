#![forbid(unsafe_code)]
//! Compatibility config and path behavior shared by rust and python entrypoints.

use std::collections::BTreeMap;
use std::hash::BuildHasher;
use std::path::{Path, PathBuf};
use std::{fs, io};

use super::io::atomic_write_text;

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
#[derive(Debug, thiserror::Error)]
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
    /// Config file contains duplicate keys.
    #[error("duplicate config key `{key}` at line {line}")]
    DuplicateConfigKey {
        /// Duplicate key.
        key: String,
        /// 1-based line number where duplicate was detected.
        line: usize,
    },
    /// Lock file already exists for mutable state operation.
    #[error("state lock is already held at {0}")]
    LockHeld(PathBuf),
    /// Underlying I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Resolve effective compatibility paths with strict precedence:
/// CLI flag overrides -> environment variables -> config file -> defaults.
pub fn discover_compatibility_paths(
    home_dir: Option<&Path>,
    cli_overrides: &PathOverrides,
    env_map: &std::collections::HashMap<String, String, impl BuildHasher>,
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

    Ok(CompatibilityPaths {
        config_file,
        history_file,
        plugins_dir,
    })
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
                if values.contains_key(trimmed_key) {
                    return Err(CompatibilityError::DuplicateConfigKey {
                        key: trimmed_key.to_string(),
                        line: line_no,
                    });
                }
                values.insert(trimmed_key.to_string(), trimmed_value.to_string());
            }
            _ => {
                return Err(CompatibilityError::UnsupportedConfigKey(
                    trimmed_key.to_string(),
                ));
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

    atomic_write_text(path, &rendered)
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

#[cfg(test)]
mod tests {
    use super::{
        parse_compatibility_config, CompatibilityError, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
        ENV_PLUGINS_PATH,
    };

    #[test]
    fn parser_rejects_duplicate_keys() {
        let source = format!(
            "{ENV_CONFIG_PATH}=a.env\n{ENV_HISTORY_PATH}=a.history\n{ENV_CONFIG_PATH}=b.env\n"
        );

        let err = parse_compatibility_config(&source).expect_err("duplicate key should fail");
        assert!(matches!(
            err,
            CompatibilityError::DuplicateConfigKey { key, line }
            if key == ENV_CONFIG_PATH && line == 3
        ));
    }

    #[test]
    fn parser_rejects_unknown_keys() {
        let source = "UNKNOWN=/tmp/path\n";
        let err = parse_compatibility_config(source).expect_err("unknown key should fail");
        assert!(matches!(err, CompatibilityError::UnsupportedConfigKey(key) if key == "UNKNOWN"));
    }

    #[test]
    fn parser_accepts_known_keys_once() {
        let source = format!(
            "{ENV_CONFIG_PATH}=cfg.env\n{ENV_HISTORY_PATH}=history.log\n{ENV_PLUGINS_PATH}=plugins\n"
        );
        let parsed = parse_compatibility_config(&source).expect("parse should pass");
        assert_eq!(
            parsed.config_file.as_deref(),
            Some(std::path::Path::new("cfg.env"))
        );
        assert_eq!(
            parsed.history_file.as_deref(),
            Some(std::path::Path::new("history.log"))
        );
        assert_eq!(
            parsed.plugins_dir.as_deref(),
            Some(std::path::Path::new("plugins"))
        );
    }
}
