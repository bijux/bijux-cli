#![forbid(unsafe_code)]
//! Filesystem storage adapter primitives.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Write text to a target path using a temp-file + rename flow.
pub fn atomic_write_text(path: &Path, content: &str) -> std::io::Result<()> {
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
            Err(err) => return Err(err),
        };

        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        return Ok(());
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!("unable to allocate unique temp file for {}", path.display()),
    ))
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
