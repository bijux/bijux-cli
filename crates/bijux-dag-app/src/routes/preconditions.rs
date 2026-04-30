use crate::ExitCode;
use std::path::Path;

fn has_unsafe_component(path: &Path) -> bool {
    path.components().any(|component| matches!(component, std::path::Component::ParentDir))
}

pub(crate) fn require_safe_path(path: &Path) -> Result<(), ExitCode> {
    if has_unsafe_component(path) {
        return Err(ExitCode::from(2));
    }
    Ok(())
}

pub(crate) fn require_existing_path(path: &Path) -> Result<(), ExitCode> {
    require_safe_path(path)?;
    if path.exists() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

pub(crate) fn require_file(path: &Path) -> Result<(), ExitCode> {
    require_safe_path(path)?;
    if path.is_file() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

pub(crate) fn require_directory(path: &Path) -> Result<(), ExitCode> {
    require_safe_path(path)?;
    if path.is_dir() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

pub(crate) fn require_run_directory(path: &Path) -> Result<(), ExitCode> {
    require_directory(path)?;
    if path.join("manifest.json").is_file() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        require_directory, require_existing_path, require_file, require_run_directory,
        require_safe_path,
    };
    use std::path::Path;

    #[test]
    fn require_existing_path_rejects_missing() {
        let missing = std::path::Path::new("/definitely-missing-path-for-bijux");
        assert!(require_existing_path(missing).is_err());
    }

    #[test]
    fn require_file_rejects_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert!(require_file(tmp.path()).is_err());
    }

    #[test]
    fn require_safe_path_rejects_parent_traversal() {
        let path = Path::new("../outside");
        assert!(require_safe_path(path).is_err());
    }

    #[test]
    fn require_directory_rejects_files() {
        let tmp = tempfile::tempdir().expect("tmp");
        let file = tmp.path().join("file.txt");
        std::fs::write(&file, b"x").expect("write");
        assert!(require_directory(&file).is_err());
    }

    #[test]
    fn require_run_directory_rejects_missing_manifest() {
        let tmp = tempfile::tempdir().expect("tmp");
        assert!(require_run_directory(tmp.path()).is_err());
    }
}
