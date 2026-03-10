#![forbid(unsafe_code)]
//! Canonical user-facing Rust binary entrypoint.

use std::process::ExitCode;

#[cfg(test)]
use bijux_cli_python as _;
#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    bijux_cli::bootstrap::run::run_cli_from_env()
}
