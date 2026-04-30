use crate::ExitCode;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;

pub(crate) fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
}
