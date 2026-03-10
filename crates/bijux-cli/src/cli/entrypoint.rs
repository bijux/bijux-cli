#![forbid(unsafe_code)]
//! CLI process entrypoint helpers.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use crate::app::run_app;
use crate::kernel::map_error_category_to_exit;

/// Execute the CLI process using current OS argv.
#[must_use]
pub fn run_cli_from_env() -> ExitCode {
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

    let result = match run_app(&argv) {
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
