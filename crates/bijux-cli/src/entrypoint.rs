#![forbid(unsafe_code)]
//! Backward-compatible re-export of process entrypoint helper.

pub use crate::bootstrap::run::run_cli_from_env;
