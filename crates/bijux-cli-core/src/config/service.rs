#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use bijux_cli_install::run_config_migrations;
use serde_json::{json, Value};

use super::error::ConfigError;
use super::storage::{ConfigRepository, FileConfigRepository};
use super::validation::{normalize_key, validate_value};

pub(crate) trait ConfigPathProvider {
    fn config_path(&self) -> &Path;
}

#[derive(Debug, Clone)]
pub(crate) struct StaticConfigPathProvider {
    config_path: PathBuf,
}

impl StaticConfigPathProvider {
    #[must_use]
    pub(crate) fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
}

impl ConfigPathProvider for StaticConfigPathProvider {
    fn config_path(&self) -> &Path {
        &self.config_path
    }
}

pub(crate) trait ConfigService {
    fn list_paths(&self, history_path: &Path, plugins_path: &Path) -> Value;
    fn get_value(&self, raw_key: &str) -> Result<Value, ConfigError>;
    fn set_pair(&self, raw_pair: &str) -> Result<Value, ConfigError>;
}

pub(crate) struct DefaultConfigService<P, R> {
    path_provider: P,
    repository: R,
}

impl<P, R> DefaultConfigService<P, R> {
    #[must_use]
    pub(crate) fn new(path_provider: P, repository: R) -> Self {
        Self { path_provider, repository }
    }
}

impl<P, R> DefaultConfigService<P, R>
where
    P: ConfigPathProvider,
    R: ConfigRepository,
{
    fn parse_set_pair(&self, raw_pair: &str) -> Result<(String, String), ConfigError> {
        if !raw_pair.contains('=') {
            return Err(ConfigError::validation("Invalid argument: KEY=VALUE required"));
        }
        let (raw_key, raw_value) = raw_pair.split_once('=').expect("contains checked");
        let key = normalize_key(raw_key)?;
        let mut value = raw_value.to_string();
        if value.len() >= 2
            && ((value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\'')))
        {
            value = super::serialization::decode_quoted_value(&value[1..value.len() - 1]);
        }
        validate_value(&value)?;
        Ok((key, value))
    }

    fn load_map(&self) -> Result<BTreeMap<String, String>, ConfigError> {
        self.repository.load(self.path_provider.config_path())
    }
}

impl ConfigService for DefaultConfigService<StaticConfigPathProvider, FileConfigRepository> {
    fn list_paths(&self, history_path: &Path, plugins_path: &Path) -> Value {
        json!({
            "BIJUXCLI_CONFIG": self.path_provider.config_path(),
            "BIJUXCLI_HISTORY_FILE": history_path,
            "BIJUXCLI_PLUGINS_DIR": plugins_path,
        })
    }

    fn get_value(&self, raw_key: &str) -> Result<Value, ConfigError> {
        let normalized_key = normalize_key(raw_key)?;
        let env_key = format!("BIJUXCLI_{}", normalized_key.to_ascii_uppercase());
        let value = if let Ok(value) = std::env::var(&env_key) {
            value
        } else {
            let values = self.load_map()?;
            values
                .get(&normalized_key)
                .cloned()
                .ok_or_else(|| ConfigError::not_found(format!("Config key not found: {raw_key}")))?
        };

        Ok(json!({
            "value": value,
            "key": normalized_key,
            "source_path": self.path_provider.config_path(),
        }))
    }

    fn set_pair(&self, raw_pair: &str) -> Result<Value, ConfigError> {
        run_config_migrations(self.path_provider.config_path(), 1)
            .map_err(|err| ConfigError::persistence(err.to_string()))?;

        let (key, value) = self.parse_set_pair(raw_pair)?;
        let mut values = self.load_map()?;
        values.insert(key.clone(), value.clone());
        self.repository.save(self.path_provider.config_path(), &values)?;
        Ok(json!({
            "status": "updated",
            "key": key,
            "value": value,
            "updated": self.path_provider.config_path(),
        }))
    }
}
