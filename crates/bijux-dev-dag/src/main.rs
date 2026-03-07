mod cli;
mod commands;
mod policy;
mod repo;
mod report;
mod suites;
mod tooling;

use std::process::ExitCode;

fn main() -> ExitCode {
    commands::entry_main()
}
