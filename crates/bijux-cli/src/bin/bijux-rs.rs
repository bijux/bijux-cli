#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use std::process::ExitCode;

#[cfg(test)]
use bijux_cli_python as _;
#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    bijux_cli::entrypoint::run_cli_from_env()
}
