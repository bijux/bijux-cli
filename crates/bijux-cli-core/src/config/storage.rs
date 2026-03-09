#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::error::ConfigError;
use super::serialization::{decode_quoted_value, render_env};
use super::validation::{normalize_key, validate_value};

pub(crate) trait ConfigRepository {
    fn load(&self, path: &Path) -> Result<BTreeMap<String, String>, ConfigError>;
    fn save(&self, path: &Path, values: &BTreeMap<String, String>) -> Result<(), ConfigError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct FileConfigRepository;

impl ConfigRepository for FileConfigRepository {
    fn load(&self, path: &Path) -> Result<BTreeMap<String, String>, ConfigError> {
        if !path.exists() {
            return Ok(BTreeMap::new());
        }
        let text = fs::read_to_string(path).map_err(|err| ConfigError::persistence(err.to_string()))?;
        let mut out = BTreeMap::new();
        for (index, raw_line) in text.lines().enumerate() {
            let line_no = index + 1;
            let trimmed = raw_line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
                return Err(ConfigError::parse(format!("Malformed line {line_no}: {raw_line}")));
            };
            let key = normalize_key(raw_key)?;
            let value = decode_quoted_value(raw_value.trim());
            validate_value(&value)?;
            out.insert(key, value);
        }
        Ok(out)
    }

    fn save(&self, path: &Path, values: &BTreeMap<String, String>) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| ConfigError::persistence(err.to_string()))?;
        }

        let rendered = render_env(values);
        let temp_path = path.with_extension("tmp");
        let mut temp = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)
            .map_err(|err| ConfigError::persistence(err.to_string()))?;
        temp.write_all(rendered.as_bytes())
            .and_then(|_| temp.sync_all())
            .map_err(|err| ConfigError::persistence(err.to_string()))?;
        fs::rename(temp_path, path).map_err(|err| ConfigError::persistence(err.to_string()))?;
        Ok(())
    }
}
