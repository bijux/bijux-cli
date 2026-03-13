#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Resolve the root directory of an installed local manifest when provenance points at
/// `plugin.manifest.json`.
pub fn installed_manifest_root(source: &str) -> Option<&Path> {
    let path = Path::new(source);
    if path.file_name().and_then(|name| name.to_str()) != Some("plugin.manifest.json") {
        return None;
    }
    path.parent()
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
pub fn resolve_delegated_entrypoint(source: &str, entrypoint: &str) -> Option<PathBuf> {
    let manifest_root = installed_manifest_root(source)?;
    delegated_entrypoint_candidates(manifest_root, entrypoint)
        .into_iter()
        .find(|candidate| candidate.exists())
}

#[cfg(test)]
mod tests {
    use super::{delegated_entrypoint_candidates, installed_manifest_root};

    use std::path::Path;

    #[test]
    fn manifest_root_only_accepts_current_manifest_filename() {
        assert!(installed_manifest_root("/tmp/example/plugin.manifest.json").is_some());
        assert!(installed_manifest_root("/tmp/example/plugin.json").is_none());
        assert!(installed_manifest_root("/tmp/example").is_none());
    }

    #[test]
    fn delegated_entrypoint_candidates_cover_module_and_package_forms() {
        let root = Path::new("/tmp/example");
        let candidates = delegated_entrypoint_candidates(root, "plugin.main:run");
        assert_eq!(candidates[0], Path::new("/tmp/example/plugin/main.py"));
        assert_eq!(candidates[1], Path::new("/tmp/example/plugin/main/__init__.py"));
    }
}
