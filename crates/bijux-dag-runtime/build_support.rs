use std::path::{Path, PathBuf};

pub(crate) const BUILD_GIT_SHA_ENV: &str = "BIJUX_DAG_BUILD_GIT_SHA";

pub(crate) fn workspace_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}

pub(crate) fn normalize_git_sha(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if !(7..=40).contains(&normalized.len()) {
        return None;
    }
    normalized.chars().all(|character| character.is_ascii_hexdigit()).then_some(normalized)
}

pub(crate) fn git_dir_from_workspace_root(workspace_root: &Path) -> Option<PathBuf> {
    let dot_git = workspace_root.join(".git");
    if dot_git.is_dir() {
        return Some(dot_git);
    }

    let git_file = std::fs::read_to_string(&dot_git).ok()?;
    let reference = git_file.trim().strip_prefix("gitdir: ")?;
    let git_dir = Path::new(reference);
    Some(if git_dir.is_absolute() { git_dir.to_path_buf() } else { workspace_root.join(git_dir) })
}

pub(crate) fn git_rerun_paths(git_dir: &Path) -> Vec<PathBuf> {
    let head_path = git_dir.join("HEAD");
    let mut paths = vec![head_path.clone(), git_dir.join("packed-refs")];

    let Ok(head_contents) = std::fs::read_to_string(&head_path) else {
        return paths;
    };
    let Some(reference) = head_contents.trim().strip_prefix("ref: ") else {
        return paths;
    };
    paths.push(git_dir.join(reference));
    paths
}
