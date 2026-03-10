#![forbid(unsafe_code)]
//! Mutable installation state helpers.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::{fs, io};

use super::compatibility::CompatibilityError;

/// Acquire process lock for mutable state operations.
pub fn acquire_state_lock(lock_path: &Path) -> Result<StateLockGuard, CompatibilityError> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(lock_path)
    {
        Ok(mut file) => {
            file.write_all(b"bijux-cli lock\n")?;
            file.sync_all()?;
            Ok(StateLockGuard {
                path: lock_path.to_path_buf(),
            })
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(CompatibilityError::LockHeld(lock_path.to_path_buf()))
        }
        Err(error) => Err(CompatibilityError::Io(error)),
    }
}

/// Guard that removes the lock path when dropped.
#[derive(Debug)]
pub struct StateLockGuard {
    path: PathBuf,
}

impl Drop for StateLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Ensure history file exists and parent directory is present.
pub fn ensure_history_file(path: &Path) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)?;
        file.write_all(b"[]\n")?;
        file.sync_all()?;
    }

    Ok(())
}

/// Ensure plugin directory exists.
pub fn ensure_plugins_dir(path: &Path) -> Result<(), CompatibilityError> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Placeholder migration entrypoint for forward config evolution.
pub fn run_config_migrations(
    _config_path: &Path,
    _current_version: u32,
) -> Result<(), CompatibilityError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::compatibility::CompatibilityError;
    use super::acquire_state_lock;

    fn make_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bijux-install-state-{name}-{nanos}"));
        fs::create_dir_all(&path).expect("mkdir");
        path
    }

    #[test]
    fn lock_contention_reports_lock_held_and_unlocks_on_drop() {
        let temp = make_temp_dir("lock-contention");
        let lock_path = temp.join("state.lock");

        let guard = acquire_state_lock(&lock_path).expect("first lock");
        let err = acquire_state_lock(&lock_path).expect_err("second lock should fail");
        assert!(matches!(err, CompatibilityError::LockHeld(_)));

        drop(guard);

        let _guard2 = acquire_state_lock(&lock_path).expect("lock should be reusable after drop");
    }
}
