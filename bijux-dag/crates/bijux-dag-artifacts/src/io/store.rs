use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactStoreSupportLevel {
    Implemented,
    ModeledOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactStoreCapabilities {
    pub support_level: ArtifactStoreSupportLevel,
    pub can_write_bytes: bool,
    pub can_read_bytes: bool,
}

pub trait ArtifactStoreBackend: Send + Sync {
    fn write_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), String>;
    fn read_bytes(&self, key: &str) -> Result<Vec<u8>, String>;
    fn capabilities(&self) -> ArtifactStoreCapabilities {
        ArtifactStoreCapabilities {
            support_level: ArtifactStoreSupportLevel::Implemented,
            can_write_bytes: true,
            can_read_bytes: true,
        }
    }
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
        Err("object store backend is modeled-only in this runtime".to_string())
    }

    fn capabilities(&self) -> ArtifactStoreCapabilities {
        ArtifactStoreCapabilities {
            support_level: ArtifactStoreSupportLevel::ModeledOnly,
            can_write_bytes: false,
            can_read_bytes: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactStoreBackend, ArtifactStoreSupportLevel, FilesystemArtifactStore,
        ObjectArtifactStore,
    };

    #[test]
    fn filesystem_artifact_store_roundtrips_bytes() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = FilesystemArtifactStore::new(dir.path());
        store
            .write_bytes("cas/aa/payload", b"hello")
            .expect("write");
        let loaded = store.read_bytes("cas/aa/payload").expect("read");
        assert_eq!(loaded, b"hello");
    }

    #[test]
    fn object_artifact_store_reports_modeled_only_capabilities() {
        let store = ObjectArtifactStore {
            bucket: "bucket".to_string(),
            prefix: "prefix".to_string(),
        };
        let caps = store.capabilities();
        assert_eq!(caps.support_level, ArtifactStoreSupportLevel::ModeledOnly);
        assert!(!caps.can_write_bytes);
        assert!(!caps.can_read_bytes);
        assert!(store.write_bytes("x", b"y").is_err());
        assert!(store.read_bytes("x").is_err());
    }
}
