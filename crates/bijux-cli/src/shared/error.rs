#![forbid(unsafe_code)]
//! Shared error aliases and wrappers.

/// Shared result alias for cross-module utilities.
pub type SharedResult<T> = anyhow::Result<T>;
