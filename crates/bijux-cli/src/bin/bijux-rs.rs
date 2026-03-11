#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use std::process::ExitCode;

#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    bijux_cli::bootstrap::run::run_cli_from_env()
}
