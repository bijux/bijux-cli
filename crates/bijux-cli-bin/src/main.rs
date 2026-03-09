#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().collect();
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
