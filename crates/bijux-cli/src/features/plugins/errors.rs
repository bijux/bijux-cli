#![forbid(unsafe_code)]

use std::io;
use std::path::PathBuf;

use crate::contracts::PluginKind;

/// Plugin manifest parsing/validation/registry errors.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// Manifest parse failed.
    #[error("plugin manifest parse failed: {0}")]
    ManifestParse(String),
    /// Missing or invalid required field.
    #[error("plugin manifest field invalid: {0}")]
    InvalidField(String),
    /// Namespace format is invalid.
    #[error("plugin namespace is invalid: {0}")]
    InvalidNamespace(String),
    /// Namespace is reserved.
    #[error("plugin namespace is reserved: {0}")]
    ReservedNamespace(String),
    /// Namespace collides with core namespace.
    #[error("plugin namespace collides with core namespace: {0}")]
    CoreNamespaceConflict(String),
    /// Namespace collides with future official product namespace.
    #[error("plugin namespace collides with reserved product namespace: {0}")]
    FutureNamespaceConflict(String),
    /// Alias duplication detected in single manifest.
    #[error("plugin manifest contains duplicate alias: {0}")]
    DuplicateAlias(String),
    /// Alias collides with an already installed plugin alias.
    #[error("plugin alias conflicts with installed plugin: {0}")]
    AliasConflict(String),
    /// Plugin compatibility does not include host version.
    #[error("plugin is incompatible with host version {host_version}")]
    IncompatibleVersion {
        /// Host version used for validation.
        host_version: String,
    },
    /// Plugin entrypoint is invalid for selected kind.
    #[error("plugin entrypoint is invalid for kind {kind:?}")]
    InvalidEntrypoint {
        /// Plugin kind.
        kind: PluginKind,
    },
    /// Plugin kind is not supported by current runtime.
    #[error("plugin kind is not supported in current runtime: {0:?}")]
    UnsupportedKind(PluginKind),
    /// Plugin namespace already exists in registry.
    #[error("plugin namespace already installed: {0}")]
    NamespaceConflict(String),
    /// Registry file is corrupted.
    #[error("plugin registry is corrupted")]
    RegistryCorrupted,
    /// Plugin not found by namespace.
    #[error("plugin not found: {0}")]
    PluginNotFound(String),
    /// Registry lock is already held by another writer.
    #[error("plugin registry lock is held at {0}")]
    RegistryLocked(PathBuf),
    /// Delegated plugin execution denied due missing capability.
    #[error("plugin is missing required capability: {0}")]
    MissingCapability(String),
    /// I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// JSON failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
