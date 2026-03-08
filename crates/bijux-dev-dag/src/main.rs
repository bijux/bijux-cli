#![allow(dead_code)]

mod cli;
mod commands;
mod policy;
mod repo;
mod report;
mod suites;
mod tooling;

use std::process::ExitCode;
#[cfg(test)]
use bijux_dag_testkit as _;
// Keep the dependency reachable at the binary root for strict target dependency checks.
use tempfile as _;

fn main() -> ExitCode {
    commands::entry_main()
}
