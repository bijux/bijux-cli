//! Workspace path resolution for dev-cli report assembly.

use std::path::{Path, PathBuf};

fn has_workspace_markers(root: &Path) -> bool {
    root.join("Cargo.toml").is_file() && root.join("crates").is_dir()
}

/// Resolve workspace root based on this crate location.
#[must_use]
pub fn workspace_root() -> PathBuf {
    let manifest_anchored = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let resolved = manifest_anchored
        .canonicalize()
        .unwrap_or(manifest_anchored);
    if !has_workspace_markers(&resolved) {
        panic!(
            "Failed to resolve workspace root from {}: expected Cargo.toml and crates/ markers",
            resolved.display()
        );
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::workspace_root;

    #[test]
    fn workspace_root_points_to_repository_root() {
        let root = workspace_root();
        assert!(
            root.join("Cargo.toml").is_file(),
            "workspace root must include Cargo.toml"
        );
        assert!(
            root.join("crates").is_dir(),
            "workspace root must include crates/"
        );
    }
}
