#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::compatibility::CompatibilityError;

/// Write text to a target path using a temp-file + rename flow.
///
/// This is the canonical state write path for install/config state files.
pub fn atomic_write_text(path: &Path, content: &str) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    for attempt in 0..64 {
        let temporary = unique_temp_path(path, attempt);
        let file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary);
        let mut file = match file {
            Ok(file) => file,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(CompatibilityError::Io(err)),
        };

        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        return Ok(());
    }

    Err(CompatibilityError::Io(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!("unable to allocate unique temp file for {}", path.display()),
    )))
}

fn unique_temp_path(path: &Path, attempt: u32) -> std::path::PathBuf {
    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state");
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let ticket = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);

    parent.join(format!(".{stem}.{pid}.{nanos}.{ticket}.{attempt}.tmp"))
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
