use std::path::{Path, PathBuf};

pub trait ArtifactStoreBackend: Send + Sync {
    fn write_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), String>;
    fn read_bytes(&self, key: &str) -> Result<Vec<u8>, String>;
}

#[derive(Debug, Clone)]
pub struct FilesystemArtifactStore {
    root: PathBuf,
}

impl FilesystemArtifactStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }
}

impl ArtifactStoreBackend for FilesystemArtifactStore {
    fn write_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), String> {
        let path = self.root.join(key);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        std::fs::write(path, bytes).map_err(|err| err.to_string())
    }

    fn read_bytes(&self, key: &str) -> Result<Vec<u8>, String> {
        std::fs::read(self.root.join(key)).map_err(|err| err.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ObjectArtifactStore {
    pub bucket: String,
    pub prefix: String,
}

impl ArtifactStoreBackend for ObjectArtifactStore {
    fn write_bytes(&self, _key: &str, _bytes: &[u8]) -> Result<(), String> {
        Err("object store backend is not implemented in this runtime".to_string())
    }

    fn read_bytes(&self, _key: &str) -> Result<Vec<u8>, String> {
        Err("object store backend is not implemented in this runtime".to_string())
    }
}
