#![forbid(unsafe_code)]
//! Compatibility binary alias.
//!
//! `bijux` is the canonical binary name.
//! `bijux-rs` remains available for existing scripts and will be removed
//! after the next stable release cycle.

use std::process::ExitCode;

#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    bijux_cli::api::runtime::run_cli_from_env()
}
