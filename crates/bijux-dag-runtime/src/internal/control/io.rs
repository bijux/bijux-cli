use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub trait Fs: Send + Sync {
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;
    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()>;
    fn open_append(&self, path: &Path) -> io::Result<fs::File>;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<fs::DirEntry>>;
    fn metadata(&self, path: &Path) -> io::Result<fs::Metadata>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;
    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64>;
    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn symlink(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn set_permissions(&self, path: &Path, perms: fs::Permissions) -> io::Result<()>;
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StdFs;

impl Fs for StdFs {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        fs::write(path, data)
    }

    fn open_append(&self, path: &Path) -> io::Result<fs::File> {
        fs::OpenOptions::new().create(true).append(true).open(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<fs::DirEntry>> {
        let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());
        Ok(entries)
    }

    fn metadata(&self, path: &Path) -> io::Result<fs::Metadata> {
        fs::metadata(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::remove_dir_all(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<u64> {
        fs::copy(from, to)
    }

    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::hard_link(from, to)
    }

    fn symlink(&self, from: &Path, to: &Path) -> io::Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(from, to)
        }
        #[cfg(not(unix))]
        {
            let _ = from;
            let _ = to;
            Err(io::Error::new(io::ErrorKind::Other, "symlink not supported"))
        }
    }

    fn set_permissions(&self, path: &Path, perms: fs::Permissions) -> io::Result<()> {
        fs::set_permissions(path, perms)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        fs::canonicalize(path)
    }
}
