#![forbid(unsafe_code)]
//! Backward-compatible re-export of process entrypoint helper.

pub use crate::cli::entrypoint::run_cli_from_env;
