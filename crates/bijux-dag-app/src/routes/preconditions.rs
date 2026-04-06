use crate::ExitCode;
use std::path::Path;

pub(crate) fn require_existing_path(path: &Path) -> Result<(), ExitCode> {
    if path.exists() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

pub(crate) fn require_file(path: &Path) -> Result<(), ExitCode> {
    if path.is_file() {
        Ok(())
    } else {
        Err(ExitCode::from(3))
    }
}

#[cfg(test)]
mod tests {
    use super::{require_existing_path, require_file};

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
}
