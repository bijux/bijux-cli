use serde_json::Value;
use std::fs;
use std::path::Path;

pub(crate) fn read_json_value(path: &Path) -> Result<Value, String> {
    let payload = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&payload).map_err(|err| err.to_string())
}

pub(crate) fn write_pretty_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let payload = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{read_json_value, write_pretty_json};

    #[test]
    fn roundtrip_json_value() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("payload.json");
        let value = serde_json::json!({"ok": true});
        write_pretty_json(&path, &value).expect("write json");
        let loaded = read_json_value(&path).expect("read json");
        assert_eq!(loaded.get("ok").and_then(|v| v.as_bool()), Some(true));
    }
}
