#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use bijux_cli_core::kernel::map_error_category_to_exit;
use bijux_cli_install as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

#[cfg(test)]
use bijux_cli_python as _;
#[cfg(test)]
use libc as _;

fn main() -> ExitCode {
    let mut argv: Vec<String> = Vec::new();
    for value in env::args_os() {
        let value = match value.into_string() {
            Ok(valid) => valid,
            Err(_) => {
                let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
                return ExitCode::from(map_error_category_to_exit("usage") as u8);
            }
        };
        argv.push(value);
    }
    let result = match bijux_cli_core::app::run_app(&argv) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    };

    if !result.stdout.is_empty() {
        let _ = write!(io::stdout(), "{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        let _ = write!(io::stderr(), "{}", result.stderr);
    }

    ExitCode::from(result.exit_code as u8)
}
