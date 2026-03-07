//! Artifact crate service interfaces.

use crate::{ArtifactError, Manifest, RunDir};

pub trait RunArtifactStore {
    fn write_manifest(&self, run_dir: &RunDir, manifest: &Manifest) -> Result<(), ArtifactError>;
}

pub trait RunArtifactVerifier {
    fn verify_run_dir(&self, run_dir: &std::path::Path) -> Result<(), ArtifactError>;
}
