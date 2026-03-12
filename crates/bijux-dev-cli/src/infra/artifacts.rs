//! Shared helpers for reading and traversing maintainer artifact inputs.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Render artifact path for payload metadata using a workspace-relative path when available.
#[must_use]
pub fn artifact_source_path(path: &Path) -> String {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let normalized_workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.clone());
    let rendered = if let Ok(relative) = path.strip_prefix(&normalized_workspace_root) {
        relative.to_path_buf()
    } else if let Ok(cwd) = std::env::current_dir() {
        path.strip_prefix(&cwd).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };
    rendered.to_string_lossy().replace('\\', "/")
}

/// Read JSON payload from disk.
///
/// On failure, returns an object carrying `_artifact_state`:
/// - `missing`
/// - `unreadable`
/// - `malformed`
#[must_use]
pub fn read_json_if_exists(path: &Path) -> Value {
    let source = artifact_source_path(path);
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(payload) => payload,
            Err(error) => json!({
                "source": source,
                "_artifact_state": "malformed",
                "_artifact_path": source,
                "_artifact_error": error.to_string(),
            }),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            json!({
                "source": source,
                "_artifact_state": "missing",
                "_artifact_path": source,
            })
        }
        Err(error) => json!({
            "source": source,
            "_artifact_state": "unreadable",
            "_artifact_path": source,
            "_artifact_error": error.to_string(),
        }),
    }
}

/// Return the normalized artifact state for a payload returned by `read_json_if_exists`.
#[must_use]
pub fn json_artifact_state(payload: &Value) -> &str {
    payload
        .get("_artifact_state")
        .and_then(Value::as_str)
        .unwrap_or("valid")
}

/// Read text payload from disk and return empty string when unavailable.
#[must_use]
pub fn read_text_if_exists(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Recursively collect all files under a base directory in deterministic order.
#[must_use]
pub fn collect_files_recursive(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

/// Render a path relative to workspace root with normalized separators.
#[must_use]
pub fn relative_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Parse makefile target names from a `makes/*.mk` style file.
#[must_use]
pub fn parse_make_targets(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for raw in text.lines() {
        if raw.starts_with('\t') || raw.starts_with('#') || raw.trim().is_empty() {
            continue;
        }
        let Some((left, _)) = raw.split_once(':') else {
            continue;
        };
        let target = left.trim();
        if target.is_empty()
            || target.contains(' ')
            || target.contains('=')
            || target.starts_with('.')
        {
            continue;
        }
        out.push(target.to_string());
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::{json_artifact_state, read_json_if_exists};

    #[test]
    fn read_json_if_exists_reports_missing_files() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-missing-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let payload = read_json_if_exists(&root.join("missing.json"));
        assert_eq!(json_artifact_state(&payload), "missing");
    }

    #[test]
    fn read_json_if_exists_reports_malformed_files() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-malformed-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("mkdir");
        let path = root.join("broken.json");
        fs::write(&path, "{not-json").expect("write");

        let payload = read_json_if_exists(&path);
        assert_eq!(json_artifact_state(&payload), "malformed");
    }

    #[test]
    fn read_json_if_exists_reports_unreadable_paths() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-unreadable-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("mkdir");

        let payload = read_json_if_exists(&root);
        assert_eq!(json_artifact_state(&payload), "unreadable");
    }

    #[test]
    fn read_json_if_exists_keeps_valid_payloads() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-valid-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("mkdir");
        let path = root.join("ok.json");
        fs::write(&path, r#"{"ok":true}"#).expect("write");

        let payload = read_json_if_exists(&path);
        assert_eq!(json_artifact_state(&payload), "valid");
        assert_eq!(payload, json!({"ok": true}));
    }
}
