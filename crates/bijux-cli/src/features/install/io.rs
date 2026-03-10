#![forbid(unsafe_code)]

use std::path::Path;

use super::compatibility::CompatibilityError;
use crate::infrastructure::fs_store;

/// Write text to a target path using a temp-file + rename flow.
///
/// This is the canonical state write path for install/config state files.
pub fn atomic_write_text(path: &Path, content: &str) -> Result<(), CompatibilityError> {
    fs_store::atomic_write_text(path, content).map_err(CompatibilityError::Io)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::thread;
    use tempfile::TempDir;

    use super::atomic_write_text;

    #[test]
    fn concurrent_writers_keep_file_readable() {
        let temp = TempDir::new().expect("tempdir");
        let target = Arc::new(temp.path().join("config.env"));
        fs::write(target.as_path(), "BIJUXCLI_ALPHA=seed\n").expect("seed");

        let mut writers = Vec::new();
        for i in 0..8 {
            let target = Arc::clone(&target);
            writers.push(thread::spawn(move || {
                for _ in 0..40 {
                    let body = format!("BIJUXCLI_ALPHA={i}\n");
                    atomic_write_text(target.as_path(), &body).expect("atomic write");
                }
            }));
        }

        for writer in writers {
            writer.join().expect("join writer");
        }

        let final_text = fs::read_to_string(target.as_path()).expect("final read");
        assert!(final_text.starts_with("BIJUXCLI_ALPHA="));
        assert!(final_text.ends_with('\n'));
        assert_eq!(final_text.lines().count(), 1);
    }
}
