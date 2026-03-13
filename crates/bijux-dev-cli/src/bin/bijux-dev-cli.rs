#![forbid(unsafe_code)]
//! Binary entrypoint for maintainer `bijux-dev-cli` process delegation.

use std::process::ExitCode;

fn main() -> ExitCode {
    bijux_dev_cli::runtime::entrypoint::run_cli_from_env()
}
