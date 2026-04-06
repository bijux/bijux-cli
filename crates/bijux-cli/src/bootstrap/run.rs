#![forbid(unsafe_code)]
//! CLI process entrypoint orchestration.

use std::io::{self, Write};
use std::process::ExitCode;

use crate::bootstrap::repl::try_run_interactive_repl;
use crate::bootstrap::wiring::{decode_os_argv, emit_run_result};
use crate::interface::cli::dispatch::run_app;
use crate::kernel::map_error_category_to_exit;
use crate::shared::telemetry::TelemetrySpan;

fn normalize_process_exit_code(code: i32) -> u8 {
    if code <= 0 {
        return u8::from(code != 0);
    }
    if code > i32::from(u8::MAX) {
        return u8::MAX;
    }
    code as u8
}

/// Execute the CLI process using current OS argv.
#[must_use]
pub fn run_cli_from_env() -> ExitCode {
    let argv = match decode_os_argv() {
        Ok(value) => value,
        Err(_) => {
            let telemetry = TelemetrySpan::start(
                "bijux-cli",
                &["bijux".to_string(), "<invalid-utf8-argv>".to_string()],
            );
            telemetry.record(
                "argv.decode.error",
                serde_json::json!({"message":"invalid UTF-8 argument in argv"}),
            );
            telemetry.finish_exit(2, 0, "invalid UTF-8 argument in argv\n".len());
            let _ = writeln!(io::stderr(), "invalid UTF-8 argument in argv");
            return ExitCode::from(map_error_category_to_exit("usage") as u8);
        }
    };

    match try_run_interactive_repl(&argv) {
        Ok(Some(code)) => return ExitCode::from(normalize_process_exit_code(code)),
        Ok(None) => {}
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    }

    let result = match run_app(&argv) {
        Ok(value) => value,
        Err(error) => {
            let _ = writeln!(io::stderr(), "{error}");
            return ExitCode::from(1);
        }
    };

    emit_run_result(&result);
    ExitCode::from(normalize_process_exit_code(result.exit_code))
}

#[cfg(test)]
mod tests {
    use super::normalize_process_exit_code;

    #[test]
    fn normalize_exit_code_clamps_negative_and_large_values() {
        assert_eq!(normalize_process_exit_code(0), 0);
        assert_eq!(normalize_process_exit_code(2), 2);
        assert_eq!(normalize_process_exit_code(-1), 1);
        assert_eq!(normalize_process_exit_code(300), u8::MAX);
    }
}
