//! Subprocess creation boundary helpers.

use std::process::{Command, Output};

pub fn output(command: &str, args: &[&str]) -> std::io::Result<Output> {
    Command::new(command).args(args).output()
}

pub fn output_with<F>(command: &str, configure: F) -> std::io::Result<Output>
where
    F: FnOnce(&mut Command),
{
    let mut cmd = Command::new(command);
    configure(&mut cmd);
    cmd.output()
}
