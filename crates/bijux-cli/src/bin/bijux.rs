#![forbid(unsafe_code)]
//! Canonical user-facing Rust binary entrypoint.

use std::process::ExitCode;

#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    bijux_cli::api::runtime::run_cli_from_env()
}
