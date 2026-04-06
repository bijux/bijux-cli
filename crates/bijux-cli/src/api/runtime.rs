#![forbid(unsafe_code)]
//! Runtime execution entrypoints.

pub use crate::bootstrap::run::run_cli_from_env;
pub use crate::interface::cli::dispatch::{run_app, AppRunResult};
