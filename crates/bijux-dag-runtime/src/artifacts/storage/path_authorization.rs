use std::path::{Path, PathBuf};

fn ensure_under_root(root: &Path, candidate: &Path, label: &str) -> Result<(), String> {
    let canonical_root = root
        .canonicalize()
        .map_err(|err| format!("cannot resolve {label} root {}: {err}", root.display()))?;
    let canonical_candidate = candidate
        .canonicalize()
        .map_err(|err| format!("cannot resolve {label} path {}: {err}", candidate.display()))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "{label} path escapes authorized root: {} not within {}",
            canonical_candidate.display(),
            canonical_root.display()
        ));
    }
    Ok(())
}

pub fn authorize_input_path(input_root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    ensure_under_root(input_root, candidate, "input")?;
    Ok(candidate.to_path_buf())
}

pub fn authorize_output_path(output_root: &Path, candidate: &Path) -> Result<PathBuf, String> {
    ensure_under_root(output_root, candidate, "output")?;
    Ok(candidate.to_path_buf())
}
