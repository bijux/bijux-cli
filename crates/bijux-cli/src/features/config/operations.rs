#![forbid(unsafe_code)]
//! Config feature operations exposed to command adapters.

use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::layered::{self, LayeredConfigOptions};
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

pub(crate) fn schema(scope: Option<&str>) -> Result<Value> {
    layered::schema_report(scope).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn validate(config_file: &Path, cwd: &Path, profile: Option<&str>) -> Result<Value> {
    layered::validate_report(config_file, cwd, profile).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn explain(
    config_file: &Path,
    cwd: &Path,
    key: &str,
    profile: Option<&str>,
    include_secrets: bool,
) -> Result<Value> {
    layered::explain_report(config_file, cwd, key, profile, include_secrets)
        .map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn repair(config_file: &Path) -> Result<Value> {
    layered::repair_report(config_file).map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn export_with_options(
    config_file: &Path,
    cwd: &Path,
    export_path: &Path,
    options: &LayeredConfigOptions,
) -> Result<Value> {
    layered::export_report(config_file, cwd, export_path, options)
        .map_err(|err| anyhow!(err.to_string()))
}

pub(crate) fn load_with_options(
    config_file: &Path,
    source_path: &Path,
    options: &LayeredConfigOptions,
) -> Result<Value> {
    layered::load_report(config_file, source_path, options)
        .map_err(|err| anyhow!("Failed to load config: {}", err))
}
