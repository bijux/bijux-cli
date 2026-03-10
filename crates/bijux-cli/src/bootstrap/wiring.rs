#![forbid(unsafe_code)]
//! Boot-time IO wiring for argv decoding and stream emission.

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};

use crate::app::AppRunResult;

pub(crate) fn decode_os_argv() -> Result<Vec<String>, OsString> {
    let mut argv = Vec::new();
    for value in env::args_os() {
        match value.into_string() {
            Ok(valid) => argv.push(valid),
            Err(invalid) => return Err(invalid),
        }
    }
    Ok(argv)
}

pub(crate) fn emit_run_result(result: &AppRunResult) {
    if !result.stdout.is_empty() {
        let _ = write!(io::stdout(), "{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        let _ = write!(io::stderr(), "{}", result.stderr);
    }
}
