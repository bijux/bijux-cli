use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

pub(super) fn newest_run(runs: &Path) -> Result<PathBuf, String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });
    candidates.into_iter().next().ok_or_else(|| format!("no runs found in {}", runs.display()))
}

pub(super) fn two_latest_runs(runs: &Path) -> Result<(PathBuf, PathBuf), String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|n| n.starts_with("run-"))
        })
        .collect();

    if candidates.len() < 2 {
        return Err(format!("expected at least 2 runs in {}", runs.display()));
    }

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });

    Ok((candidates[0].clone(), candidates[1].clone()))
}

pub(super) fn wildcard_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut index = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !pattern.starts_with('*') {
            if !text[index..].starts_with(part) {
                return false;
            }
            index += part.len();
            continue;
        }
        if i == parts.len() - 1 && !pattern.ends_with('*') {
            return text.ends_with(part);
        }
        if let Some(found) = text[index..].find(part) {
            index += found + part.len();
        } else {
            return false;
        }
    }
    true
}

pub(super) fn is_transient_component(component: &str) -> bool {
    matches!(
        component,
        ".git" | "target" | "artifacts" | "node_modules" | ".venv" | "venv" | "build" | "dist"
    )
}

pub(super) fn is_transient_path(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    relative
        .components()
        .any(|component| component.as_os_str().to_str().is_some_and(is_transient_component))
}

pub(super) fn collect_all_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            if path.file_name().and_then(|v| v.to_str()) == Some(".git") {
                continue;
            }
            collect_all_files(&path, out)?;
            continue;
        }
        if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn tracked_files_with_extension(root: &Path, ext: &str) -> Option<Vec<PathBuf>> {
    let top_level = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()
        .ok()?;
    if !top_level.status.success() {
        return None;
    }
    let repository_root = PathBuf::from(String::from_utf8_lossy(&top_level.stdout).trim());
    if repository_root.canonicalize().ok()? != root.canonicalize().ok()? {
        return None;
    }

    let glob = format!("*.{ext}");
    let output =
        Command::new("git").args(["ls-files", "--", &glob]).current_dir(root).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| root.join(line))
        .collect();
    Some(files)
}

pub(super) fn repository_files_with_extension(
    root: &Path,
    ext: &str,
) -> Result<Vec<PathBuf>, String> {
    if let Some(files) = tracked_files_with_extension(root, ext) {
        return Ok(files);
    }

    let mut files = Vec::new();
    collect_all_files(root, &mut files)?;
    files.retain(|path| {
        !is_transient_path(root, path)
            && path.extension().and_then(|value| value.to_str()) == Some(ext)
    });
    Ok(files)
}

pub(super) fn collect_files_with_extension(
    dir: &Path,
    ext: &str,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)
        .map_err(|err| format!("failed to read directory {}: {err}", dir.display()))?
    {
        let path = entry
            .map_err(|err| format!("failed to read entry in {}: {err}", dir.display()))?
            .path();
        if path.is_dir() {
            collect_files_with_extension(&path, ext, out)?;
            continue;
        }
        if path.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        collect_files_with_extension, is_transient_path, repository_files_with_extension,
        wildcard_match,
    };
    use std::fs;

    #[test]
    fn wildcard_matching_covers_star_and_question() {
        assert!(wildcard_match("runtime/*.rs", "runtime/mod.rs"));
        assert!(wildcard_match("runtime/*.rs", "runtime/sub/mod.rs"));
        assert!(wildcard_match("run-*", "run-abc"));
        assert!(!wildcard_match("runtime/*.rs", "runtime/mod.md"));
    }

    #[test]
    fn extension_collection_ignores_missing_dirs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut files = Vec::new();
        collect_files_with_extension(&temp.path().join("missing"), "rs", &mut files)
            .expect("collect");
        assert!(files.is_empty());

        let nested = temp.path().join("nested");
        fs::create_dir_all(&nested).expect("create dir");
        fs::write(nested.join("a.rs"), "fn main() {}").expect("write rs");
        fs::write(nested.join("b.md"), "# docs").expect("write md");

        collect_files_with_extension(temp.path(), "rs", &mut files).expect("collect");
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("a.rs"));
    }

    #[test]
    fn transient_path_detection_marks_run_outputs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_file = temp.path().join("artifacts/run-123/report.json");
        fs::create_dir_all(artifact_file.parent().expect("artifact parent")).expect("create dir");
        fs::write(&artifact_file, "{}").expect("write artifact file");

        assert!(is_transient_path(temp.path(), &artifact_file));
    }

    #[test]
    fn repository_extension_collection_ignores_transient_fallback_dirs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let governed = temp.path().join("configs/policy.json");
        let artifact = temp.path().join("artifacts/run-123/policy.json");
        fs::create_dir_all(governed.parent().expect("governed parent")).expect("create governed");
        fs::create_dir_all(artifact.parent().expect("artifact parent")).expect("create artifact");
        fs::write(&governed, "{}").expect("write governed file");
        fs::write(&artifact, "{}").expect("write artifact file");

        let files = repository_files_with_extension(temp.path(), "json").expect("collect files");
        assert_eq!(files, vec![governed]);
    }
}
