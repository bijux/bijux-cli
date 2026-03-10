#![forbid(unsafe_code)]
//! Process adapter primitives.

use std::io::{self, Write};

/// Write a payload to stdout.
pub fn write_stdout(payload: &str) {
    let _ = write!(io::stdout(), "{payload}");
}

/// Write a payload to stderr.
pub fn write_stderr(payload: &str) {
    let _ = write!(io::stderr(), "{payload}");
}
