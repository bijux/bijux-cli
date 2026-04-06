#![forbid(unsafe_code)]
//! Environment adapter primitives.

/// Read a process environment variable.
#[must_use]
pub fn var(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Return process argv encoded as UTF-8 where possible.
pub fn argv_utf8() -> Result<Vec<String>, std::ffi::OsString> {
    let mut argv = Vec::new();
    for token in std::env::args_os() {
        match token.into_string() {
            Ok(value) => argv.push(value),
            Err(invalid) => return Err(invalid),
        }
    }
    Ok(argv)
}
