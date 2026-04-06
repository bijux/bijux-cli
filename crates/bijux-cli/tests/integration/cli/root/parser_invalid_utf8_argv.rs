#![forbid(unsafe_code)]
//! Invalid UTF-8 argv hardening coverage.
//! test_type: parser-fuzz

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

#[cfg(unix)]
#[test]
fn malformed_utf8_argv_is_rejected_without_panic() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    use std::process::Command;

    let bad = OsString::from_vec(vec![0x66, 0x6f, 0x80, 0xff, 0x6f]);
    let output =
        Command::new(env!("CARGO_BIN_EXE_bijux")).arg(bad).output().expect("binary should execute");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    assert!(
        stderr.to_lowercase().contains("invalid utf-8")
            || stderr.to_lowercase().contains("invalid utf8"),
        "stderr should report invalid utf-8 argv, got: {stderr}"
    );
}

#[cfg(not(unix))]
#[test]
fn malformed_utf8_argv_coverage_not_supported_on_this_platform() {
    // Windows argv is UTF-16 from the OS, so invalid UTF-8 bytes are not representable this way.
    assert!(true);
}
