#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

fn is_current_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".manifest.json"))
}

/// Resolve the root directory of an installed local manifest when the stored manifest path or
/// backwards-compatible provenance points at a current manifest file.
pub fn installed_manifest_root(manifest_path: Option<&str>, source: &str) -> Option<PathBuf> {
    let candidate = manifest_path.unwrap_or(source);
    let path = Path::new(candidate);
    if !is_current_manifest_path(path) {
        return None;
    }
    path.parent().map(Path::to_path_buf)
}

/// Candidate filesystem locations for a delegated/Python entrypoint module.
pub fn delegated_entrypoint_candidates(manifest_root: &Path, entrypoint: &str) -> Vec<PathBuf> {
    let module = entrypoint.split_once(':').map_or(entrypoint, |(module, _)| module);
    if module.trim().is_empty() {
        return Vec::new();
    }

    let segments: Option<Vec<&str>> = module
        .split('.')
        .map(str::trim)
        .map(|segment| if segment.is_empty() { None } else { Some(segment) })
        .collect();
    let Some(segments) = segments else {
        return Vec::new();
    };

    let mut file_path = manifest_root.to_path_buf();
    for segment in &segments {
        file_path.push(segment);
    }
    if file_path.extension().is_none() {
        file_path.set_extension("py");
    }

    let mut package_path = manifest_root.to_path_buf();
    for segment in &segments {
        package_path.push(segment);
    }
    package_path.push("__init__.py");

    if file_path == package_path {
        vec![file_path]
    } else {
        vec![file_path, package_path]
    }
}

/// Resolve an existing delegated/Python entrypoint path when local manifest provenance is known.
pub fn resolve_delegated_entrypoint(
    manifest_path: Option<&str>,
    source: &str,
    entrypoint: &str,
) -> Option<PathBuf> {
    let manifest_root = installed_manifest_root(manifest_path, source)?;
    delegated_entrypoint_candidates(&manifest_root, entrypoint)
        .into_iter()
        .find(|candidate| candidate.exists())
}

/// Resolve an external executable entrypoint relative to the installed manifest when provenance is local.
pub fn resolve_external_exec_entrypoint(
    manifest_path: Option<&str>,
    source: &str,
    entrypoint: &str,
) -> PathBuf {
    let entrypoint_path = Path::new(entrypoint);
    if entrypoint_path.is_absolute() {
        return entrypoint_path.to_path_buf();
    }

    if let Some(manifest_root) = installed_manifest_root(manifest_path, source) {
        return manifest_root.join(entrypoint_path);
    }

    entrypoint_path.to_path_buf()
}

#[cfg(unix)]
pub fn is_executable(path: &Path) -> std::io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;

    Ok(path.exists()
        && path.is_file()
        && std::fs::metadata(path)?.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
pub fn is_executable(path: &Path) -> std::io::Result<bool> {
    Ok(path.exists() && path.is_file())
}

#[cfg(test)]
mod tests {
    use super::{
        delegated_entrypoint_candidates, installed_manifest_root, resolve_external_exec_entrypoint,
    };

    use std::path::Path;

    #[test]
    fn manifest_root_only_accepts_current_manifest_filename() {
        assert!(installed_manifest_root(
            Some("/tmp/example/plugin.manifest.json"),
            "custom-source"
        )
        .is_some());
        assert!(installed_manifest_root(
            Some("/tmp/example/custom.manifest.json"),
            "custom-source"
        )
        .is_some());
        assert!(
            installed_manifest_root(Some("/tmp/example/plugin.json"), "custom-source").is_none()
        );
        assert!(installed_manifest_root(Some("/tmp/example"), "custom-source").is_none());
    }

    #[test]
    fn manifest_root_falls_back_to_legacy_source_for_existing_registry_entries() {
        assert!(installed_manifest_root(None, "/tmp/example/plugin.manifest.json").is_some());
        assert!(installed_manifest_root(None, "/tmp/example/plugin.json").is_none());
    }

    #[test]
    fn delegated_entrypoint_candidates_cover_module_and_package_forms() {
        let root = Path::new("/tmp/example");
        let candidates = delegated_entrypoint_candidates(root, "plugin.main:run");
        assert_eq!(candidates[0], Path::new("/tmp/example/plugin/main.py"));
        assert_eq!(candidates[1], Path::new("/tmp/example/plugin/main/__init__.py"));
    }

    #[test]
    fn external_exec_entrypoint_uses_manifest_root_for_relative_paths() {
        let resolved = resolve_external_exec_entrypoint(
            Some("/tmp/example/plugin.manifest.json"),
            "custom-source",
            "bin/run.sh",
        );
        assert_eq!(resolved, Path::new("/tmp/example/bin/run.sh"));
    }
}
