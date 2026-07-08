use bijux_dag_artifacts::paths::is_normalized_relative_path;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

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

pub(crate) fn authorize_declared_output_target(
    output_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, String> {
    if !is_normalized_relative_path(relative_path) {
        return Err(format!("output path must be normalized and relative: {relative_path}"));
    }

    let canonical_root = output_root
        .canonicalize()
        .map_err(|err| format!("cannot resolve output root {}: {err}", output_root.display()))?;
    let mut current = canonical_root.clone();

    for component in Path::new(relative_path).components() {
        let Component::Normal(part) = component else {
            return Err(format!("output path must be normalized and relative: {relative_path}"));
        };
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("output path traverses symlink: {relative_path}"));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => break,
            Err(error) => {
                return Err(format!(
                    "cannot inspect output path component {}: {error}",
                    current.display()
                ));
            }
        }
    }

    Ok(output_root.join(relative_path))
}

#[cfg(test)]
mod tests {
    use super::{authorize_declared_output_target, authorize_output_path};
    use std::fs;

    #[test]
    fn declared_output_target_accepts_missing_normalized_child_under_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output_root = dir.path().join("output");
        fs::create_dir_all(&output_root).expect("output root");

        let target =
            authorize_declared_output_target(&output_root, "node/result.txt").expect("target");
        assert_eq!(target, output_root.join("node").join("result.txt"));
    }

    #[test]
    fn declared_output_target_rejects_parent_traversal_and_absolute_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output_root = dir.path().join("output");
        fs::create_dir_all(&output_root).expect("output root");

        assert!(authorize_declared_output_target(&output_root, "../escape.txt").is_err());
        assert!(authorize_declared_output_target(&output_root, "/escape.txt").is_err());
    }

    #[test]
    fn declared_output_target_rejects_symlinked_existing_parent() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let dir = tempfile::tempdir().expect("tempdir");
            let output_root = dir.path().join("output");
            let outside = dir.path().join("outside");
            fs::create_dir_all(&output_root).expect("output root");
            fs::create_dir_all(&outside).expect("outside");
            symlink(&outside, output_root.join("escape")).expect("symlink");

            let error = authorize_declared_output_target(&output_root, "escape/result.txt")
                .expect_err("symlinked parent must fail");
            assert!(error.contains("traverses symlink"));
        }
    }

    #[test]
    fn existing_output_authorization_still_rejects_escape() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output_root = dir.path().join("output");
        fs::create_dir_all(&output_root).expect("output root");
        let outside = dir.path().join("outside.txt");
        fs::write(&outside, "x").expect("outside");

        assert!(authorize_output_path(&output_root, &outside).is_err());
    }
}
