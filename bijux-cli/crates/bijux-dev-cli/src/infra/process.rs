//! Process execution adapter helpers for maintainer workflows.

use std::process::{Command, Output};

/// Run a command and collect process output.
pub fn run(command: &mut Command) -> std::io::Result<Output> {
    command.output()
}
