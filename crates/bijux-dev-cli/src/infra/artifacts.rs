//! Shared helpers for reading and traversing maintainer artifact inputs.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Render artifact path for payload metadata using a workspace-relative path when available.
#[must_use]
pub fn artifact_source_path(path: &Path) -> String {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let normalized_workspace_root =
        workspace_root.canonicalize().unwrap_or_else(|_| workspace_root.clone());
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
    payload.get("_artifact_state").and_then(Value::as_str).unwrap_or("valid")
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

/// Replace a destination directory with a copied snapshot of a source directory tree.
pub fn replace_directory_tree(source: &Path, destination: &Path) -> std::io::Result<usize> {
    if !source.is_dir() {
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("source directory does not exist: {}", source.display()),
        ));
    }

    if destination.exists() {
        fs::remove_dir_all(destination)?;
    }

    copy_directory_tree(source, destination)
}

fn copy_directory_tree(source: &Path, destination: &Path) -> std::io::Result<usize> {
    let mut copied_file_count = 0usize;
    let mut stack = vec![(source.to_path_buf(), destination.to_path_buf())];

    while let Some((source_dir, destination_dir)) = stack.pop() {
        fs::create_dir_all(&destination_dir)?;

        let mut entries = Vec::new();
        for entry in fs::read_dir(&source_dir)? {
            entries.push(entry?);
        }
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let source_path = entry.path();
            let destination_path = destination_dir.join(entry.file_name());
            let file_type = entry.file_type()?;

            if file_type.is_dir() {
                stack.push((source_path, destination_path));
                continue;
            }

            if file_type.is_file() {
                fs::copy(&source_path, &destination_path)?;
                copied_file_count += 1;
                continue;
            }

            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("unsupported filesystem entry: {}", source_path.display()),
            ));
        }
    }

    Ok(copied_file_count)
}

/// Render a path relative to workspace root with normalized separators.
#[must_use]
pub fn relative_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
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
    use std::io::ErrorKind;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::json;

    use super::{json_artifact_state, read_json_if_exists, replace_directory_tree};

    #[test]
    fn read_json_if_exists_reports_missing_files() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-missing-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        let payload = read_json_if_exists(&root.join("missing.json"));
        assert_eq!(json_artifact_state(&payload), "missing");
    }

    #[test]
    fn read_json_if_exists_reports_malformed_files() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-malformed-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
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
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(&root).expect("mkdir");

        let payload = read_json_if_exists(&root);
        assert_eq!(json_artifact_state(&payload), "unreadable");
    }

    #[test]
    fn read_json_if_exists_keeps_valid_payloads() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-valid-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(&root).expect("mkdir");
        let path = root.join("ok.json");
        fs::write(&path, r#"{"ok":true}"#).expect("write");

        let payload = read_json_if_exists(&path);
        assert_eq!(json_artifact_state(&payload), "valid");
        assert_eq!(payload, json!({"ok": true}));
    }

    #[test]
    fn replace_directory_tree_replaces_existing_destination_contents() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-copy-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        let source = root.join("source");
        let destination = root.join("destination");

        fs::create_dir_all(source.join("nested")).expect("mkdir");
        fs::write(source.join("root.txt"), "root").expect("write");
        fs::write(source.join("nested/child.txt"), "child").expect("write");

        fs::create_dir_all(&destination).expect("mkdir");
        fs::write(destination.join("stale.txt"), "stale").expect("write");

        let copied_file_count = replace_directory_tree(&source, &destination).expect("copy");

        assert_eq!(copied_file_count, 2);
        assert!(!destination.join("stale.txt").exists());
        assert_eq!(fs::read_to_string(destination.join("root.txt")).expect("read"), "root");
        assert_eq!(
            fs::read_to_string(destination.join("nested/child.txt")).expect("read"),
            "child"
        );
    }

    #[test]
    fn replace_directory_tree_reports_missing_source_directory() {
        let root = std::env::temp_dir().join(format!(
            "bijux-artifacts-copy-missing-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        let error = replace_directory_tree(&root.join("missing"), &root.join("destination"))
            .expect_err("missing");

        assert_eq!(error.kind(), ErrorKind::NotFound);
    }
}
