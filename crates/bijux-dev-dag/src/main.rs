mod cli;
mod commands;
mod policy;
mod repo;
mod report;
mod suites;
mod tooling;

use std::process::ExitCode;
use tempfile as _;

fn main() -> ExitCode {
    commands::entry_main()
}
