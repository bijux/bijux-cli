#![forbid(unsafe_code)]
//! Compatibility shim for legacy CLI entrypoint module path.

pub use crate::bootstrap::run::run_cli_from_env;
