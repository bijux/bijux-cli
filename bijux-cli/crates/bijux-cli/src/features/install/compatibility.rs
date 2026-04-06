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
    /// Config file contains an empty path value.
    #[error("empty config value for `{key}` at line {line}")]
    EmptyConfigValue {
        /// Config key.
        key: String,
        /// 1-based line number where empty value was detected.
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
    let defaults = home_dir.map(default_compatibility_paths);

    let config_file = select_path(
        cli_overrides.config_file.as_ref(),
        env_map.get(ENV_CONFIG_PATH),
        file_config.config_file.as_ref(),
        defaults.as_ref().map(|paths| paths.config_file.as_path()),
        home_dir,
    )?;
    let history_file = select_path(
        cli_overrides.history_file.as_ref(),
        env_map.get(ENV_HISTORY_PATH),
        file_config.history_file.as_ref(),
        defaults.as_ref().map(|paths| paths.history_file.as_path()),
        home_dir,
    )?;
    let plugins_dir = select_path(
        cli_overrides.plugins_dir.as_ref(),
        env_map.get(ENV_PLUGINS_PATH),
        file_config.plugins_dir.as_ref(),
        defaults.as_ref().map(|paths| paths.plugins_dir.as_path()),
        home_dir,
    )?;

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
                if trimmed_value.is_empty() {
                    return Err(CompatibilityError::EmptyConfigValue {
                        key: trimmed_key.to_string(),
                        line: line_no,
                    });
                }
                if values.contains_key(trimmed_key) {
                    return Err(CompatibilityError::DuplicateConfigKey {
                        key: trimmed_key.to_string(),
                        line: line_no,
                    });
                }
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
    default_value: Option<&Path>,
    home_dir: Option<&Path>,
) -> Result<PathBuf, CompatibilityError> {
    let candidate = cli_value
        .filter(|value| !path_is_empty(value))
        .cloned()
        .or_else(|| env_value.filter(|value| !value.trim().is_empty()).map(PathBuf::from))
        .or_else(|| config_value.filter(|value| !path_is_empty(value)).cloned())
        .or_else(|| default_value.map(Path::to_path_buf))
        .ok_or(CompatibilityError::MissingHome)?;

    normalize_path(&candidate, home_dir)
}

fn path_is_empty(path: &Path) -> bool {
    path.to_str().is_some_and(|value| value.trim().is_empty())
}

fn normalize_path(path: &Path, home_dir: Option<&Path>) -> Result<PathBuf, CompatibilityError> {
    let Some(raw) = path.to_str() else {
        return Ok(path.to_path_buf());
    };

    if raw == "~" {
        return home_dir.map(Path::to_path_buf).ok_or(CompatibilityError::MissingHome);
    }
    if let Some(tail) = raw.strip_prefix("~/") {
        let home = home_dir.ok_or(CompatibilityError::MissingHome)?;
        return Ok(home.join(tail));
    }
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }

    let home = home_dir.ok_or(CompatibilityError::MissingHome)?;
    Ok(home.join(path))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::{
        discover_compatibility_paths, parse_compatibility_config, CompatibilityConfig,
        CompatibilityError, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
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
        assert_eq!(parsed.config_file.as_deref(), Some(std::path::Path::new("cfg.env")));
        assert_eq!(parsed.history_file.as_deref(), Some(std::path::Path::new("history.log")));
        assert_eq!(parsed.plugins_dir.as_deref(), Some(std::path::Path::new("plugins")));
    }

    #[test]
    fn parser_rejects_empty_values() {
        let source = format!("{ENV_HISTORY_PATH}=\n");
        let err = parse_compatibility_config(&source).expect_err("empty value should fail");
        assert!(matches!(
            err,
            CompatibilityError::EmptyConfigValue { key, line } if key == ENV_HISTORY_PATH && line == 1
        ));
    }

    #[test]
    fn discover_paths_ignores_empty_overrides_and_uses_defaults() {
        let home = PathBuf::from("/tmp/bijux-compat-home");
        let overrides = PathOverrides {
            config_file: Some(PathBuf::from("")),
            history_file: Some(PathBuf::from("   ")),
            plugins_dir: None,
        };
        let mut env_map = HashMap::new();
        env_map.insert(ENV_CONFIG_PATH.to_string(), " ".to_string());
        env_map.insert(ENV_HISTORY_PATH.to_string(), "".to_string());
        env_map.insert(ENV_PLUGINS_PATH.to_string(), "\t".to_string());

        let resolved = discover_compatibility_paths(
            Some(home.as_path()),
            &overrides,
            &env_map,
            &CompatibilityConfig::default(),
        )
        .expect("resolve");

        assert_eq!(resolved.config_file, home.join(".bijux/.env"));
        assert_eq!(resolved.history_file, home.join(".bijux/.history"));
        assert_eq!(resolved.plugins_dir, home.join(".bijux/.plugins"));
    }

    #[test]
    fn discover_paths_without_home_supports_absolute_overrides() {
        let overrides = PathOverrides {
            config_file: Some(PathBuf::from("/tmp/bijux/config.env")),
            history_file: Some(PathBuf::from("/tmp/bijux/history.log")),
            plugins_dir: Some(PathBuf::from("/tmp/bijux/plugins")),
        };

        let resolved = discover_compatibility_paths(
            None,
            &overrides,
            &HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect("absolute overrides should not require home");

        assert_eq!(resolved.config_file, PathBuf::from("/tmp/bijux/config.env"));
        assert_eq!(resolved.history_file, PathBuf::from("/tmp/bijux/history.log"));
        assert_eq!(resolved.plugins_dir, PathBuf::from("/tmp/bijux/plugins"));
    }

    #[test]
    fn discover_paths_without_home_rejects_defaults() {
        let error = discover_compatibility_paths(
            None,
            &PathOverrides::default(),
            &HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect_err("missing home should fail when defaults are required");
        assert!(matches!(error, CompatibilityError::MissingHome));
    }

    #[test]
    fn discover_paths_without_home_rejects_relative_overrides() {
        let overrides = PathOverrides {
            config_file: Some(PathBuf::from("config.env")),
            history_file: Some(PathBuf::from("/tmp/bijux/history.log")),
            plugins_dir: Some(PathBuf::from("/tmp/bijux/plugins")),
        };

        let error = discover_compatibility_paths(
            None,
            &overrides,
            &HashMap::new(),
            &CompatibilityConfig::default(),
        )
        .expect_err("relative overrides still need home to normalize");
        assert!(matches!(error, CompatibilityError::MissingHome));
    }
}
