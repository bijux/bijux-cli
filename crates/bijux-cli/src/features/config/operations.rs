#![forbid(unsafe_code)]
//! Config feature operations exposed to command adapters.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::features::config::service::{
    ConfigService, DefaultConfigService, StaticConfigPathProvider,
};
use crate::features::config::storage::FileConfigRepository;

fn config_service(
    config_file: &Path,
) -> DefaultConfigService<StaticConfigPathProvider, FileConfigRepository> {
    DefaultConfigService::new(
        StaticConfigPathProvider::new(config_file.to_path_buf()),
        FileConfigRepository,
    )
}

pub(crate) fn list_entries(config_file: &Path) -> Result<Value> {
    config_service(config_file).list_entries().map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn get_value(config_file: &Path, key: &str) -> Result<Value> {
    config_service(config_file).get_value(key).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn set_pair(config_file: &Path, pair: &str) -> Result<Value> {
    config_service(config_file).set_pair(pair).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn unset_key(config_file: &Path, key: &str) -> Result<Value> {
    config_service(config_file).unset_key(key).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn clear_all(config_file: &Path) -> Result<Value> {
    config_service(config_file).clear_all().map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn reload(config_file: &Path) -> Result<Value> {
    config_service(config_file).reload().map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn export_to(config_file: &Path, export_path: &Path) -> Result<Value> {
    config_service(config_file).export_to(export_path).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn load_from(config_file: &Path, source_path: &Path) -> Result<Value> {
    config_service(config_file)
        .load_from(source_path)
        .map_err(|err| anyhow!("Failed to load config: {}", err))
}
