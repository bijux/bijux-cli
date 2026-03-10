#![forbid(unsafe_code)]
//! CLI process entrypoint orchestration.

use std::io::{self, Write};
use std::process::ExitCode;

use crate::bootstrap::wiring::{decode_os_argv, emit_run_result};
use crate::interface::cli::dispatch::run_app;
use crate::kernel::map_error_category_to_exit;

/// Execute the CLI process using current OS argv.
#[must_use]
pub fn run_cli_from_env() -> ExitCode {
    let argv = match decode_os_argv() {
        Ok(value) => value,
        Err(_) => {
            let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
            return ExitCode::from(map_error_category_to_exit("usage") as u8);
        }
    };

    let result = match run_app(&argv) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    };

    emit_run_result(&result);
    ExitCode::from(result.exit_code as u8)
}
