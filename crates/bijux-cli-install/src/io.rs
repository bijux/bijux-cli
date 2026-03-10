#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::Path;

use crate::compatibility::CompatibilityError;

/// Write text to a target path using a temp-file + rename flow.
///
/// This is the canonical state write path for install/config state files.
pub fn atomic_write_text(path: &Path, content: &str) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temporary = path.with_extension("tmp");
    {
        let mut file =
            fs::OpenOptions::new().create(true).truncate(true).write(true).open(&temporary)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    fs::rename(temporary, path)?;
    Ok(())
}
